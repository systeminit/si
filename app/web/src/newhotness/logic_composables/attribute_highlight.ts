import { ref, computed } from 'vue';
import { AttributePath } from '@/api/sdf/dal/component';

// Global state for highlighted attributes
const highlightedAttributePaths = ref<Set<string>>(new Set());
const highlightTimeouts = ref<Map<string, NodeJS.Timeout>>(new Map());
const attributeChangeData = ref<Map<string, { beforeValue: any; afterValue: any; operation: string }>>(new Map());
const currentExpandedLogKey = ref<string | null>(null);

export function useAttributeHighlight() {
  // Function to highlight an attribute by path with optional change data
  const highlightAttribute = (path: string, changeData?: { beforeValue: any; afterValue: any; operation?: string }, duration?: number, logKey?: string) => {
    console.log('📍 Attempting to highlight attribute:', path, { changeData, logKey });
    
    // Clear any existing timeout for this path
    const existingTimeout = highlightTimeouts.value.get(path);
    if (existingTimeout) {
      clearTimeout(existingTimeout);
      highlightTimeouts.value.delete(path);
    }
    
    // Add to highlighted set
    highlightedAttributePaths.value.add(path);
    
    // Store change data if provided
    if (changeData) {
      attributeChangeData.value.set(path, {
        beforeValue: changeData.beforeValue,
        afterValue: changeData.afterValue,
        operation: changeData.operation || 'update'
      });
      console.log('💾 Stored change data for path:', path, changeData);
    }
    
    // Store the expanded log key for click-outside handling
    if (logKey) {
      currentExpandedLogKey.value = logKey;
    }
    
    // Auto-scroll to the highlighted attribute
    scrollToAttribute(path);
    
    // Set timeout to remove highlight only if duration is specified
    if (duration) {
      const timeout = setTimeout(() => {
        highlightedAttributePaths.value.delete(path);
        highlightTimeouts.value.delete(path);
      }, duration);
      
      highlightTimeouts.value.set(path, timeout);
    }
  };

  // Function to clear a specific highlight
  const clearHighlight = (path: string) => {
    const timeout = highlightTimeouts.value.get(path);
    if (timeout) {
      clearTimeout(timeout);
      highlightTimeouts.value.delete(path);
    }
    highlightedAttributePaths.value.delete(path);
    attributeChangeData.value.delete(path);
    currentExpandedLogKey.value = null;
  };

  // Function to clear all highlights
  const clearAllHighlights = () => {
    // Clear all timeouts
    highlightTimeouts.value.forEach(timeout => clearTimeout(timeout));
    highlightTimeouts.value.clear();
    
    // Clear highlighted paths and change data
    highlightedAttributePaths.value.clear();
    attributeChangeData.value.clear();
    currentExpandedLogKey.value = null;
  };

  // Check if a specific path is highlighted
  const isAttributeHighlighted = (path: string) => {
    return computed(() => highlightedAttributePaths.value.has(path));
  };

  // Get all highlighted paths
  const getHighlightedPaths = () => {
    return computed(() => new Set(highlightedAttributePaths.value));
  };

  // Get change data for a specific path
  const getAttributeChangeData = (path: string) => {
    return computed(() => attributeChangeData.value.get(path));
  };
  
  // Get the currently expanded log key
  const getCurrentExpandedLogKey = () => {
    return computed(() => currentExpandedLogKey.value);
  };

  return {
    highlightAttribute,
    clearHighlight,
    clearAllHighlights,
    isAttributeHighlighted,
    getHighlightedPaths,
    getAttributeChangeData,
    getCurrentExpandedLogKey,
  };
}

// Helper function to extract property path from audit log metadata
export function extractPropertyPathFromAuditLog(log: any): string | null {
  if (!log.metadata) {
    console.log('🔴 No metadata in audit log:', log);
    return null;
  }
  
  console.log('🧾 Extracting path from audit log metadata:', log.metadata);
  
  // First, try the new attributePath field which should contain the full path
  if (log.metadata.attributePath && typeof log.metadata.attributePath === 'string') {
    let path = log.metadata.attributePath;
    console.log('🔵 Raw attributePath from metadata:', path);
    
    // Handle different path formats:
    // - Convert "root/domain/..." to "/domain/..."
    // - Ensure paths start with "/domain"
    if (path.startsWith('root/domain/')) {
      path = path.replace('root/domain/', '/domain/');
    } else if (path.startsWith('root/')) {
      path = path.replace('root/', '/domain/');
    } else if (!path.startsWith('/domain')) {
      // Handle paths that don't start with /domain (like array items)
      if (path.startsWith('/')) {
        path = `/domain${path}`;
      } else {
        path = `/domain/${path}`;
      }
    }
    
    // Normalize double slashes
    path = path.replace(/\/+/g, '/');
    
    console.log('🔵 Normalized attributePath:', path);
    return path;
  }
  
  // Try the entityName field which should now contain the full path
  if (log.entityName && typeof log.entityName === 'string' && log.entityName.startsWith('/domain')) {
    let path = log.entityName;
    console.log('🔵 Using entityName as path:', path);
    
    // Normalize the path
    path = path.replace(/\/+/g, '/');
    return path;
  }
  
  // Fallback to legacy logic for older audit logs
  const possiblePaths = [
    log.metadata.propertyPath,
    log.metadata.propName,
    log.metadata.propertyName,
    log.metadata.path,
  ];
  
  for (const path of possiblePaths) {
    if (path && typeof path === 'string') {
      let cleanPath = path;
      
      // Ensure the path starts with /domain if it doesn't already
      if (cleanPath.startsWith('/')) {
        cleanPath = cleanPath.startsWith('/domain') ? cleanPath : `/domain${cleanPath}`;
      } else {
        cleanPath = `/domain/${cleanPath}`;
      }
      
      // Normalize double slashes
      cleanPath = cleanPath.replace(/\/+/g, '/');
      
      return cleanPath;
    }
  }
  
  // Final fallback: construct basic path from prop name
  if (log.metadata.propName || log.metadata.propertyName) {
    const propName = log.metadata.propName || log.metadata.propertyName;
    return `/domain/${propName}`;
  }
  
  console.log('🔴 No valid path found in audit log');
  return null;
}

// Helper function to scroll to a highlighted attribute
function scrollToAttribute(path: string) {
  // Use nextTick to ensure the DOM has updated with the highlighting
  import('vue').then(({ nextTick }) => {
    nextTick(() => {
      // Try to find the element with retry logic for freshly created attributes
      findElementWithRetry(path, 0);
    });
  });
}

// Helper function to find element with retry logic
function findElementWithRetry(path: string, attempt = 0) {
  const maxAttempts = 5;
  const retryDelay = 100; // 100ms delay between attempts
  
  console.log(`🔍 Searching for attribute path (attempt ${attempt + 1}/${maxAttempts}):`, path);
  
  // Try different selector strategies to find the attribute element
  const selectors = [
    // Exact match
    `[data-attribute-path="${path}"]`,
    // Try with different path formats
    `[data-attribute-path="${path.replace('/domain/', 'root/domain/')}"]`,
    `[data-attribute-path="${path.replace('/domain', '/root/domain')}"]`,
  ];
  
  let element: HTMLElement | null = null;
  let foundSelector = '';
  
  for (const selector of selectors) {
    try {
      element = document.querySelector(selector) as HTMLElement;
      if (element) {
        foundSelector = selector;
        break;
      }
    } catch (e) {
      // Selector might not be supported, continue to next
      continue;
    }
  }
  
  // If we didn't find exact match, try partial matching and array format conversion
  if (!element) {
    console.log('🔍 Exact match failed, trying advanced matching...');
    const allElements = document.querySelectorAll('[data-attribute-path]');
    
    // Convert audit log path format to DOM path format
    // /domain/SsmAssociations/0/AssociationParameters/2/Key
    // → /domain/SsmAssociations/SsmAssociationsItem[0]/AssociationParameters/AssociationParametersItem[2]/Key
    const convertedPath = convertAuditLogPathToDOMPath(path);
    console.log('🔄 Converted path:', path, '→', convertedPath);
    
    // Try the converted path
    if (convertedPath !== path) {
      element = document.querySelector(`[data-attribute-path="${convertedPath}"]`) as HTMLElement;
      if (element) {
        foundSelector = `converted path: ${convertedPath}`;
      }
    }
    
    // If still not found, try fuzzy matching
    if (!element) {
      const pathElements = Array.from(allElements).filter(el => {
        const elementPath = el.getAttribute('data-attribute-path');
        if (!elementPath) return false;
        
        // Try various matching strategies
        return (
          elementPath === path ||
          elementPath === convertedPath ||
          doPathsMatch(elementPath, path) ||
          elementPath.endsWith(path.split('/').slice(-2).join('/')) ||
          path.includes(elementPath.split('/').slice(-2).join('/'))
        );
      });
      
      if (pathElements.length > 0) {
        element = pathElements[0] as HTMLElement;
        foundSelector = `fuzzy match: ${element.getAttribute('data-attribute-path')}`;
      }
    }
  }
  
  // If we found an element, scroll to it with smooth behavior
  if (element) {
    console.log('✅ Found element with selector:', foundSelector);
    element.scrollIntoView({
      behavior: 'smooth',
      block: 'center',
      inline: 'nearest'
    });
    
    // Add a temporary extra highlight effect
    element.style.transition = 'box-shadow 0.3s ease';
    element.style.boxShadow = '0 0 20px rgba(59, 130, 246, 0.5)';
    
    setTimeout(() => {
      element!.style.boxShadow = '';
    }, 2000);
  } else if (attempt < maxAttempts - 1) {
    // Element not found, retry after a short delay
    console.log(`⏳ Element not found, retrying in ${retryDelay}ms...`);
    setTimeout(() => {
      findElementWithRetry(path, attempt + 1);
    }, retryDelay);
  } else {
    // Max attempts reached, show error
    console.warn('❌ Could not find element for attribute path after', maxAttempts, 'attempts:', path);
    console.log('Available paths in DOM:');
    const allPaths = Array.from(document.querySelectorAll('[data-attribute-path]'))
      .map(el => el.getAttribute('data-attribute-path'))
      .filter(Boolean);
    console.log(allPaths.slice(0, 10)); // Log first 10 for debugging
    
    // Show a notification that the attribute no longer exists
    showMissingAttributeNotification(path);
  }
}

// Convert audit log path format to DOM path format
// Example: /domain/SsmAssociations/0/AssociationParameters/2/Key
// → /domain/SsmAssociations/SsmAssociationsItem[0]/AssociationParameters/AssociationParametersItem[2]/Key
function convertAuditLogPathToDOMPath(auditPath: string): string {
  const parts = auditPath.split('/');
  const convertedParts = [];
  
  for (let i = 0; i < parts.length; i++) {
    const part = parts[i];
    
    // Handle empty parts (can happen with leading/trailing slashes)
    if (!part) {
      convertedParts.push(part);
      continue;
    }
    
    // If this part is a number, it's an array index
    if (/^\d+$/.test(part)) {
      // Look at the previous part to determine the item name pattern
      const prevPart = parts[i - 1];
      if (prevPart) {
        // Convert plural to singular + Item pattern
        const itemName = convertToItemName(prevPart);
        convertedParts.push(`${itemName}[${part}]`);
      } else {
        convertedParts.push(part);
      }
    } else {
      convertedParts.push(part);
    }
  }
  
  return convertedParts.join('/');
}

// Convert collection names to item names
// Examples: SsmAssociations → SsmAssociationsItem, AssociationParameters → AssociationParametersItem
function convertToItemName(collectionName: string): string {
  // Common patterns in the system
  const patterns = [
    { suffix: 's', replacement: 'Item' }, // plural → Item
    { suffix: 'es', replacement: 'Item' }, // classes → ClassesItem
  ];
  
  for (const pattern of patterns) {
    if (collectionName.endsWith(pattern.suffix)) {
      return collectionName + pattern.replacement;
    }
  }
  
  // Fallback: just add Item
  return `${collectionName}Item`;
}

// Check if two paths match semantically (handling different array formats)
function doPathsMatch(domPath: string, auditPath: string): boolean {
  // Quick exact match
  if (domPath === auditPath) return true;
  
  // Convert both to a normalized format for comparison
  const normalizePath = (path: string) => {
    return path
      // Convert Item[0] → /0/
      .replace(/([A-Za-z]+)Item\[(\d+)\]/g, '/$2/')
      // Normalize multiple slashes
      .replace(/\/+/g, '/')
      // Remove trailing slash
      .replace(/\/$/, '');
  };
  
  const normalizedDomPath = normalizePath(domPath);
  const normalizedAuditPath = normalizePath(auditPath);
  
  console.log('🔄 Path matching:', {
    domPath,
    auditPath,
    normalizedDomPath,
    normalizedAuditPath,
    match: normalizedDomPath === normalizedAuditPath
  });
  
  return normalizedDomPath === normalizedAuditPath;
}

// Helper function to show notification when attribute is missing
function showMissingAttributeNotification(path: string) {
  // Create a temporary notification
  const notification = document.createElement('div');
  notification.className = 'fixed top-4 right-4 bg-yellow-100 dark:bg-yellow-900 border border-yellow-300 dark:border-yellow-700 text-yellow-800 dark:text-yellow-200 px-4 py-2 rounded shadow-lg z-50';
  notification.innerHTML = `
    <div class="flex items-center gap-2">
      <span class="text-yellow-600 dark:text-yellow-400">⚠️</span>
      <div>
        <div class="font-medium">Attribute not found</div>
        <div class="text-sm opacity-75">Path: ${path}</div>
        <div class="text-xs opacity-60">This attribute may have been deleted or restructured</div>
      </div>
    </div>
  `;
  
  document.body.appendChild(notification);
  
  // Remove after 5 seconds
  setTimeout(() => {
    if (notification.parentNode) {
      notification.parentNode.removeChild(notification);
    }
  }, 5000);
}