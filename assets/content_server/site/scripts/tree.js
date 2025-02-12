
function buildTree(paths) {
  let tree = {};
  
  paths.forEach(path => {
    let parts = path.split('/');
    let current = tree;
    
    parts.forEach((part, index) => {
      if (!current[part]) {
        current[part] = index === parts.length - 1 ? path : {};
      }
      current = current[part];
    });
  });
  return tree;
}

function renderTree(node, parentElement, isRoot = false) {
    let ul = document.createElement('ul');
    if (!isRoot) ul.classList.add('hidden');
    parentElement.appendChild(ul);
    
    let entries = Object.keys(node);
    entries.sort((a, b) => {
      let isAFolder = typeof node[a] === 'object';
      let isBFolder = typeof node[b] === 'object';
      return isAFolder === isBFolder ? a.localeCompare(b) : isBFolder - isAFolder;
    });
    
    entries.forEach(key => {
      let li = document.createElement('li');
      ul.appendChild(li);
      
      if (typeof node[key] === 'object') {
        let span = document.createElement('span');
        span.textContent = key;
        span.classList.add('folder', 'toggle');
        span.onclick = () => {
          let childUl = span.nextElementSibling;
          childUl.classList.toggle('hidden');
          span.classList.toggle('expanded');
        };
        li.appendChild(span);
        renderTree(node[key], li);
      } else {
        let a = document.createElement('a');
        a.textContent = key;
        a.href = `/${node[key]}`;
        a.target = "_blank";
        li.appendChild(a);
      }
    });
}

async function fileTree() {
  let response = await fetch('/file_list');
  let filePaths = await response.json();
  let tree = buildTree(filePaths);
  renderTree(tree, document.getElementById('fileTree'), true);
}

