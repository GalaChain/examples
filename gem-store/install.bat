@echo off
echo 🚀 Installing Gem Store Dependencies...

REM Install root dependencies
echo 📦 Installing root dependencies...
npm install

REM Install frontend dependencies
echo 📦 Installing frontend dependencies...
cd frontend
npm install
cd ..

REM Install backend dependencies
echo 📦 Installing backend dependencies...
cd backend
npm install
cd ..

echo ✅ Installation complete!
echo.
echo 📝 Next steps:
echo 1. Copy frontend/env.example to frontend/.env
echo 2. Run 'npm run dev' to start both servers
echo 3. Open http://localhost:3000 in your browser
echo.
echo 🔗 Make sure you have MetaMask installed and some GALA tokens!
pause
