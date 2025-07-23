// Print My Bridge GUI - Fixed Implementation with Tauri API checks
console.log('Print My Bridge GUI loaded');

// Global variables
let currentConfig = null;

// DOM elements
let statusDiv, tokenInput, hostInput, portInput, maxFileSizeInput, rateLimitInput;
let copyButton, generateButton, saveButton;
let autoStartCheckbox, minimizeToTrayCheckbox;

// Check if Tauri API is available
function isTauriAvailable() {
    return typeof window !== 'undefined' && 
           window.__TAURI__ && 
           typeof window.__TAURI__.core === 'object' &&
           typeof window.__TAURI__.core.invoke === 'function';
}

// Wait for Tauri to be ready
function waitForTauri(maxAttempts = 50) {
    return new Promise((resolve, reject) => {
        let attempts = 0;
        
        const checkTauri = () => {
            attempts++;
            
            if (isTauriAvailable()) {
                console.log('Tauri API is ready');
                console.log('Available Tauri modules:', Object.keys(window.__TAURI__));
                resolve();
            } else if (attempts >= maxAttempts) {
                console.error('Tauri API structure:', window.__TAURI__);
                reject(new Error('Tauri API not available after maximum attempts'));
            } else {
                console.log(`Waiting for Tauri API... attempt ${attempts}`);
                setTimeout(checkTauri, 100);
            }
        };
        
        checkTauri();
    });
}

// Initialize the application
document.addEventListener('DOMContentLoaded', async function() {
    console.log('DOM loaded, initializing...');
    
    // Get DOM elements
    statusDiv = document.getElementById('status');
    tokenInput = document.getElementById('token');
    hostInput = document.getElementById('host');
    portInput = document.getElementById('port');
    maxFileSizeInput = document.getElementById('max-file-size');
    rateLimitInput = document.getElementById('rate-limit');
    copyButton = document.getElementById('copy-token');
    generateButton = document.getElementById('generate-token');
    saveButton = document.getElementById('save-config');
    autoStartCheckbox = document.getElementById('auto-start');
    minimizeToTrayCheckbox = document.getElementById('minimize-to-tray');
    
    // Set up event listeners
    setupEventListeners();
    setupTabNavigation();
    
    try {
        // Wait for Tauri to be ready
        await waitForTauri();
        
        // Load configuration and check status
        await loadConfiguration();
        await checkBridgeStatus();
        
        // Set up periodic status check
        setInterval(checkBridgeStatus, 5000);
    } catch (error) {
        console.error('Failed to initialize Tauri:', error);
        statusDiv.textContent = '❌ Tauri API not available: ' + error.message;
        statusDiv.className = 'status-error';
        tokenInput.value = 'Tauri API not available';
    }
});

function setupTabNavigation() {
    const tabButtons = document.querySelectorAll('.tab-button');
    const tabContents = document.querySelectorAll('.tab-content');
    
    tabButtons.forEach(button => {
        button.addEventListener('click', () => {
            const targetTab = button.getAttribute('data-tab');
            
            // Remove active class from all buttons and contents
            tabButtons.forEach(btn => btn.classList.remove('active'));
            tabContents.forEach(content => content.classList.remove('active'));
            
            // Add active class to clicked button and corresponding content
            button.classList.add('active');
            document.getElementById(targetTab + '-tab').classList.add('active');
        });
    });
}

function setupEventListeners() {
    // Copy token button
    copyButton.addEventListener('click', async function() {
        try {
            if (!isTauriAvailable()) {
                throw new Error('Tauri API not available');
            }
            
            const token = tokenInput.value;
            if (token && token !== 'Loading token...' && token !== 'Tauri API not available') {
                // Use Tauri v2 clipboard API with proper error handling
                if (window.__TAURI__.clipboardManager && window.__TAURI__.clipboardManager.writeText) {
                    await window.__TAURI__.clipboardManager.writeText(token);
                    showNotification('Token copied to clipboard!', 'success');
                } else {
                    // Fallback to core invoke if clipboardManager is not available
                    await window.__TAURI__.core.invoke('plugin:clipboard-manager|write_text', { text: token });
                    showNotification('Token copied to clipboard!', 'success');
                }
            } else {
                showNotification('No token to copy', 'warning');
            }
        } catch (error) {
            console.error('Error copying token:', error);
            showNotification('Failed to copy token: ' + error.message, 'error');
        }
    });
    
    // Generate new token button
    generateButton.addEventListener('click', async function() {
        try {
            if (!isTauriAvailable()) {
                throw new Error('Tauri API not available');
            }
            
            generateButton.disabled = true;
            generateButton.textContent = '🔄 Generating...';
            
            const newToken = await window.__TAURI__.core.invoke('generate_new_token');
            tokenInput.value = newToken;
            
            showNotification('New token generated successfully!', 'success');
        } catch (error) {
            console.error('Error generating token:', error);
            showNotification('Failed to generate new token: ' + error.message, 'error');
        } finally {
            generateButton.disabled = false;
            generateButton.textContent = '🔄 Generate New Token';
        }
    });
    
    // Auto-start checkbox
    autoStartCheckbox.addEventListener('change', async function() {
        try {
            await window.__TAURI__.core.invoke('toggle_auto_start', { enable: autoStartCheckbox.checked });
            showNotification(
                autoStartCheckbox.checked ? 
                'Auto-start enabled' : 
                'Auto-start disabled', 
                'success'
            );
        } catch (error) {
            console.error('Error toggling auto-start:', error);
            showNotification('Failed to change auto-start setting: ' + error.message, 'error');
            // Revert checkbox state
            autoStartCheckbox.checked = !autoStartCheckbox.checked;
        }
    });
    
    // Save configuration button
    saveButton.addEventListener('click', async function() {
        try {
            if (!isTauriAvailable()) {
                throw new Error('Tauri API not available');
            }
            
            saveButton.disabled = true;
            saveButton.textContent = '💾 Saving...';
            
            const updatedConfig = {
                ...currentConfig,
                host: hostInput.value || '127.0.0.1',
                port: parseInt(portInput.value) || 8765,
                max_file_size_mb: parseInt(maxFileSizeInput.value) || 50,
                rate_limit_per_minute: parseInt(rateLimitInput.value) || 60,
                auto_start: autoStartCheckbox.checked,
                minimize_to_tray: minimizeToTrayCheckbox.checked
            };
            
            await window.__TAURI__.core.invoke('update_config', { config: updatedConfig });
            currentConfig = updatedConfig;
            
            showNotification('Configuration saved successfully!\nRestart required for changes to take effect.', 'success');
        } catch (error) {
            console.error('Error saving configuration:', error);
            showNotification('Failed to save configuration: ' + error.message, 'error');
        } finally {
            saveButton.disabled = false;
            saveButton.textContent = '💾 Save Configuration';
        }
    });
}

async function loadConfiguration() {
    try {
        if (!isTauriAvailable()) {
            throw new Error('Tauri API not available');
        }
        
        const config = await window.__TAURI__.core.invoke('get_config');
        currentConfig = config;
        
        // Update UI with current configuration
        hostInput.value = config.host || '127.0.0.1';
        portInput.value = config.port || 8765;
        maxFileSizeInput.value = config.max_file_size_mb || 50;
        rateLimitInput.value = config.rate_limit_per_minute || 60;
        tokenInput.value = config.api_token || 'No token available';
        autoStartCheckbox.checked = config.auto_start || false;
        minimizeToTrayCheckbox.checked = config.minimize_to_tray !== false;
        
        console.log('Configuration loaded:', config);
    } catch (error) {
        console.error('Error loading configuration:', error);
        statusDiv.textContent = 'Error loading configuration: ' + error.message;
        statusDiv.className = 'status-error';
        tokenInput.value = 'Error loading token';
    }
}

async function checkBridgeStatus() {
    try {
        if (!isTauriAvailable()) {
            throw new Error('Tauri API not available');
        }
        
        const status = await window.__TAURI__.core.invoke('get_bridge_status');
        
        if (status.active) {
            statusDiv.textContent = `✅ Bridge is running on port ${status.port} (v${status.version})`;
            statusDiv.className = 'status-success';
        } else {
            statusDiv.textContent = `❌ Bridge is not running (configured port: ${status.port})`;
            statusDiv.className = 'status-error';
        }
        
        console.log('Bridge status:', status);
    } catch (error) {
        console.error('Error checking bridge status:', error);
        statusDiv.textContent = '⚠️ Unable to check bridge status: ' + error.message;
        statusDiv.className = 'status-warning';
    }
}

function showNotification(message, type) {
    // Simple notification system
    const notification = document.createElement('div');
    notification.className = `notification notification-${type}`;
    notification.textContent = message;
    
    document.body.appendChild(notification);
    
    // Auto remove after 3 seconds
    setTimeout(() => {
        if (notification.parentNode) {
            notification.parentNode.removeChild(notification);
        }
    }, 3000);
}
