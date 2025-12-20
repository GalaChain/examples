// Simple GalaChain Wallet Connection - Plain JavaScript
// Import BrowserConnectClient from @gala-chain/connect
import { BrowserConnectClient } from '@gala-chain/connect';

let client = null;
let isConnected = false;
let walletAddress = '';

// Check if MetaMask is available
function checkMetaMask() {
    if (typeof window.ethereum === 'undefined') {
        showStatus('MetaMask is not installed. Please install MetaMask extension.', 'error');
        return false;
    }
    return true;
}

// Show status message
function showStatus(message, type = '') {
    const statusDiv = document.getElementById('status');
    statusDiv.textContent = message;
    statusDiv.className = 'status ' + type;
    statusDiv.style.display = 'block';
}

// Connect wallet function
window.connectWallet = async function() {
    if (!checkMetaMask()) {
        return;
    }

    try {
        showStatus('Connecting to wallet...', '');
        document.getElementById('connectBtn').disabled = true;

        // Create new client instance
        client = new BrowserConnectClient();
        
        // Connect to MetaMask
        const connected = await client.connect();
        
        if (connected && client.galaChainAddress) {
            walletAddress = client.galaChainAddress;
            isConnected = true;
            
            // Update UI
            document.getElementById('connectBtn').style.display = 'none';
            document.getElementById('disconnectBtn').style.display = 'block';
            document.getElementById('address').style.display = 'block';
            document.getElementById('address').innerHTML = '<strong>Connected Address:</strong><br>' + walletAddress;
            
            showStatus('Wallet connected successfully!', 'success');
        } else {
            throw new Error('Failed to connect wallet');
        }
    } catch (error) {
        console.error('Error connecting wallet:', error);
        showStatus('Error: ' + (error.message || 'Failed to connect wallet'), 'error');
        document.getElementById('connectBtn').disabled = false;
    }
};

// Disconnect wallet function
window.disconnectWallet = function() {
    if (client) {
        client.disconnect();
        client = null;
    }
    
    isConnected = false;
    walletAddress = '';
    
    // Update UI
    document.getElementById('connectBtn').style.display = 'block';
    document.getElementById('connectBtn').disabled = false;
    document.getElementById('disconnectBtn').style.display = 'none';
    document.getElementById('address').style.display = 'none';
    
    showStatus('Wallet disconnected', '');
};

// Check MetaMask on page load
if (typeof window.ethereum === 'undefined') {
    showStatus('MetaMask is not detected. Please install MetaMask extension to use this app.', 'error');
    document.getElementById('connectBtn').disabled = true;
}

