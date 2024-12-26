<template>
  <div class="pixel-board-container">
    <h2>Community Pixel Board</h2>
    <div class="canvas-container">
      <canvas 
        ref="canvas"
        :width="CANVAS_SIZE"
        :height="CANVAS_SIZE"
        @mousedown="startDrawing"
        @mousemove="draw"
        @mouseup="stopDrawing"
        @mouseleave="stopDrawing"
      ></canvas>
    </div>
    
    <div class="controls">
      <div class="color-picker">
        <input type="color" v-model="selectedColor" />
        <span>Selected Color</span>
      </div>
      
      <div class="cost-info">
        <span>Cost per pixel: 1 GALA</span>
      </div>

      <div class="burn-preview" v-if="modifiedPixels.size > 0">
        Total cost: {{ modifiedPixels.size }} GALA
      </div>

      <button 
        @click="saveCanvas" 
        :disabled="!hasChanges || isProcessing"
      >
        {{ isProcessing ? 'Processing...' : 'Burn GALA and Save' }}
      </button>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, onMounted } from 'vue'
import type { MetamaskConnectClient } from '@gala-chain/connect'
import { initializeApp } from 'firebase/app'
import { getDatabase, ref as dbRef, set, onValue, get } from 'firebase/database'

// Initialize Firebase
const firebaseConfig = {
  apiKey: import.meta.env.VITE_FIREBASE_API_KEY,
  projectId: import.meta.env.VITE_FIREBASE_PROJECT_ID,
  databaseURL: import.meta.env.VITE_FIREBASE_DATABASE_URL,
}

console.log('Firebase config:', {
  projectId: firebaseConfig.projectId,
  databaseURL: firebaseConfig.databaseURL
})

const app = initializeApp(firebaseConfig)
const db = getDatabase(app)
console.log('Firebase initialized:', {
  dbRef: dbRef(db, 'pixelboard').toString()
})

const CANVAS_SIZE = 500
const PIXEL_SIZE = 10
const GRID_SIZE = CANVAS_SIZE / PIXEL_SIZE

const props = defineProps<{
  walletAddress: string
  metamaskClient: MetamaskConnectClient
}>()

const canvas = ref<HTMLCanvasElement | null>(null)
const ctx = ref<CanvasRenderingContext2D | null>(null)
const isDrawing = ref(false)
const selectedColor = ref('#000000')
const hasChanges = ref(false)
const isProcessing = ref(false)
const modifiedPixels = ref(new Set<string>())

interface PixelData {
  x: number
  y: number
  color: string
}

const pixels = ref<Record<string, PixelData>>({})

onMounted(async () => {
  if (!canvas.value) return
  ctx.value = canvas.value.getContext('2d')
  if (!ctx.value) return
  
  drawGrid()
  
  // Listen for pixel updates
  const pixelsRef = dbRef(db, 'pixels')
  onValue(pixelsRef, 
    (snapshot) => {
      const data = snapshot.val()
      if (data) {
        loadPixels(data)
      }
    },
    (error) => {
      console.error('Firebase read error:', error)
    }
  )
})

function drawGrid() {
  if (!ctx.value) return
  
  ctx.value.strokeStyle = '#ddd'
  ctx.value.beginPath()
  
  for (let i = 0; i <= CANVAS_SIZE; i += PIXEL_SIZE) {
    ctx.value.moveTo(i, 0)
    ctx.value.lineTo(i, CANVAS_SIZE)
    ctx.value.moveTo(0, i)
    ctx.value.lineTo(CANVAS_SIZE, i)
  }
  
  ctx.value.stroke()
}

function startDrawing(event: MouseEvent) {
  isDrawing.value = true
  draw(event)
}

function draw(event: MouseEvent) {
  if (!isDrawing.value || !ctx.value || !canvas.value) return
  
  const rect = canvas.value.getBoundingClientRect()
  const x = Math.floor((event.clientX - rect.left) / PIXEL_SIZE) * PIXEL_SIZE
  const y = Math.floor((event.clientY - rect.top) / PIXEL_SIZE) * PIXEL_SIZE
  
  ctx.value.fillStyle = selectedColor.value
  ctx.value.fillRect(x, y, PIXEL_SIZE, PIXEL_SIZE)
  
  // Track modified pixels with their colors
  const key = `${x},${y}`
  pixels.value[key] = {
    x,
    y,
    color: selectedColor.value
  }
  modifiedPixels.value.add(key)
  hasChanges.value = true
}

function stopDrawing() {
  isDrawing.value = false
}

async function saveCanvas() {
  if (!ctx.value || !canvas.value || modifiedPixels.value.size === 0) return
  
  isProcessing.value = true
  
  try {
    // Calculate GALA cost (1 GALA per pixel)
    const burnAmount = modifiedPixels.value.size
    
    // Create burn transaction
    const burnTokensDto = {
      owner: props.walletAddress,
      tokenInstances: [{
        quantity: burnAmount.toString(),
        tokenInstanceKey: {
          collection: "GALA",
          category: "Unit",
          type: "none",
          additionalKey: "none",
          instance: "0"
        }
      }],
      uniqueKey: `january-2025-pixel-board-example-${Date.now()}`
    }

    // Sign and submit burn transaction
    const signedBurnDto = await props.metamaskClient.sign("BurnTokens", burnTokensDto)
    const response = await fetch(`${import.meta.env.VITE_BURN_GATEWAY_API}/BurnTokens`, {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify(signedBurnDto)
    })

    if (!response.ok) {
      throw new Error('Failed to burn tokens')
    }

    // Get current pixels from Firebase
    const pixelsRef = dbRef(db, 'pixels')
    const snapshot = await get(pixelsRef)
    const existingPixels = snapshot.val() || {}

    // Merge existing pixels with new ones
    const updates = {
      ...existingPixels,
      ...Object.fromEntries(
        Array.from(modifiedPixels.value).map(key => [key, pixels.value[key]])
      )
    }
    
    // Save merged pixels back to Firebase
    await set(pixelsRef, updates)
    console.log('Pixels saved successfully')
    
    // Clear tracking
    modifiedPixels.value.clear()
    hasChanges.value = false
    
  } catch (error) {
    console.error('Failed to save canvas:', error)
    alert('Failed to save changes')
  } finally {
    isProcessing.value = false
  }
}

function loadPixels(data: Record<string, PixelData>) {
  if (!ctx.value) return
  
  try {
    // Draw each pixel from the database
    Object.values(data).forEach(pixel => {
      ctx.value!.fillStyle = pixel.color
      ctx.value!.fillRect(pixel.x, pixel.y, PIXEL_SIZE, PIXEL_SIZE)
    })
    
    // Store loaded pixels in local state
    pixels.value = data
    
    // Redraw grid
    drawGrid()
    
    console.log('Pixels loaded successfully')
  } catch (error) {
    console.error('Failed to load pixels:', error)
  }
}
</script>

<style scoped>
.pixel-board-container {
  margin: 20px 0;
  padding: 20px;
  border: 1px solid #ddd;
  border-radius: 8px;
}

.canvas-container {
  margin: 20px 0;
  display: flex;
  justify-content: center;
}

canvas {
  border: 1px solid #ccc;
  cursor: crosshair;
}

.controls {
  display: flex;
  gap: 20px;
  align-items: center;
  justify-content: center;
  margin: 20px 0;
}

.color-picker {
  display: flex;
  flex-direction: column;
  align-items: center;
  gap: 5px;
}

.ipfs-info {
  text-align: center;
  margin-top: 10px;
}

.ipfs-info a {
  color: #666;
  text-decoration: none;
}

.ipfs-info a:hover {
  text-decoration: underline;
}

.error-message {
  color: #ff4444;
  text-align: center;
  margin: 10px 0;
  padding: 10px;
  background: #ffeeee;
  border-radius: 4px;
}

.burn-preview {
  background: #f5f5f5;
  padding: 8px 16px;
  border-radius: 4px;
  font-weight: bold;
}
</style> 