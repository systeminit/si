// SVG SETUP
// ===========
var svg = document.getElementById('svg');
svg.ns = svg.namespaceURI;

// MOUSE SETUP
// =============
var mouse = {
  currentInput: null,
  createPath: function(a, b){
    var diff = {
      x: b.x - a.x,
      y: b.y - a.y
    };
  
    var pathStr = 'M' + a.x + ',' + a.y + ' ';
    pathStr += 'C';
    pathStr += a.x + diff.x / 3 * 2 + ',' + a.y + ' ';
    pathStr += a.x + diff.x / 3 + ',' + b.y + ' ';
    pathStr += b.x + ',' + b.y;
    
    return pathStr;
  }
};

window.onmousemove = function(e){
  if(mouse.currentInput){
    var p = mouse.currentInput.path;
    var iP = mouse.currentInput.getAttachPoint();
    var oP = {x: e.pageX, y: e.pageY};
    var s = mouse.createPath(iP, oP);
    p.setAttributeNS(null, 'd', s);
  }
};

window.onclick = function(e){
  if(mouse.currentInput){
    mouse.currentInput.path.removeAttribute('d');
    if(mouse.currentInput.node){
      mouse.currentInput.node.detachInput(mouse.currentInput);
    }
    mouse.currentInput = null;
  }
};

// CLEAN UP AND ACTUAL CODE [WIP]
// ================================

function GetFullOffset(element){
  var offset = {
    top: element.offsetTop,
    left: element.offsetLeft,
  };
  
  if(element.offsetParent){
    var po = GetFullOffset(element.offsetParent);
    offset.top += po.top;
    offset.left += po.left;
    return offset;
  }
  else
    return offset;
}

function Node(name){
  // DOM Element creation
  this.domElement = document.createElement('div');
  this.domElement.classList.add('node');
  this.domElement.setAttribute('title', name);
  
  // Create output visual
  var outDom = document.createElement('span');
  outDom.classList.add('output');
  outDom.innerHTML = '&nbsp;';
  this.domElement.appendChild(outDom);
  
  // Output Click handler
  var that = this;
  outDom.onclick = function(e){
    if(mouse.currentInput &&
       !that.ownsInput(mouse.currentInput)){
      var tmp = mouse.currentInput;
      mouse.currentInput = null;
      that.connectTo(tmp);
    }
    e.stopPropagation();
  };
  
  // Node Stuffs
  this.value = '';
  this.inputs = [];
  this.connected = false;
  
  // SVG Connectors
  this.attachedPaths = [];
}

function NodeInput(name){
  this.name = name;
  this.node = null;
  
  // The dom element, here is where we could add
  // different input types
  this.domElement = document.createElement('div');
  this.domElement.innerHTML = name;
  this.domElement.classList.add('connection');
  this.domElement.classList.add('empty');
    
  // SVG Connector
  this.path = document.createElementNS(svg.ns, 'path');
  this.path.setAttributeNS(null, 'stroke', '#8e8e8e');
  this.path.setAttributeNS(null, 'stroke-width', '2');
  this.path.setAttributeNS(null, 'fill', 'none');
  svg.appendChild(this.path);
  
  // DOM Event handlers
  var that = this;
  this.domElement.onclick = function(e){
    if(mouse.currentInput){
      if(mouse.currentInput.path.hasAttribute('d'))
        mouse.currentInput.path.removeAttribute('d');
      if(mouse.currentInput.node){
        mouse.currentInput.node.detachInput(mouse.currentInput);
        mouse.currentInput.node = null;
      }
    }
    mouse.currentInput = that;
    if(that.node){
      that.node.detachInput(that);
      that.domElement.classList.remove('filled');
      that.domElement.classList.add('empty');
    }
    e.stopPropagation();
  };
}

NodeInput.prototype.getAttachPoint = function(){
  var offset = GetFullOffset(this.domElement);
  return {
    x: offset.left + this.domElement.offsetWidth - 2,
    y: offset.top + this.domElement.offsetHeight / 2
  };
};

Node.prototype.getOutputPoint = function(){
  var tmp = this.domElement.firstElementChild;
  var offset = GetFullOffset(tmp);
  return {
    x: offset.left + tmp.offsetWidth / 2,
    y: offset.top + tmp.offsetHeight / 2
  };
};

Node.prototype.addInput = function(name){
  var input = new NodeInput(name);
  this.inputs.push(input);
  this.domElement.appendChild(input.domElement);
  
  return input;
};

Node.prototype.detachInput = function(input){
  var index = -1;
  for(var i = 0; i < this.attachedPaths.length; i++){
    if(this.attachedPaths[i].input == input)
      index = i;
  };
  
  if(index >= 0){
    this.attachedPaths[index].path.removeAttribute('d');
    this.attachedPaths[index].input.node = null;
    this.attachedPaths.splice(index, 1);
  }
  
  if(this.attachedPaths.length <= 0){
    this.domElement.classList.remove('connected');
  }
};

Node.prototype.ownsInput = function(input){
  for(var i = 0; i < this.inputs.length; i++){
    if(this.inputs[i] == input)
      return true;
  }
  return false;
};

Node.prototype.updatePosition = function(){
  var outPoint = this.getOutputPoint();
  
  var aPaths = this.attachedPaths;
  for(var i = 0; i < aPaths.length; i++){
    var iPoint = aPaths[i].input.getAttachPoint();
    var pathStr = this.createPath(iPoint, outPoint);
    aPaths[i].path.setAttributeNS(null, 'd', pathStr);
  }
  
  for(var j = 0; j < this.inputs.length; j++){
    if(this.inputs[j].node != null){
      var iP = this.inputs[j].getAttachPoint();
      var oP = this.inputs[j].node.getOutputPoint();
      
      var pStr = this.createPath(iP, oP);
      this.inputs[j].path.setAttributeNS(null, 'd', pStr);
    }
  }
};

Node.prototype.createPath = function(a, b){
  var diff = {
    x: b.x - a.x,
    y: b.y - a.y
  };
  
  var pathStr = 'M' + a.x + ',' + a.y + ' ';
  pathStr += 'C';
  pathStr += a.x + diff.x / 3 * 2 + ',' + a.y + ' ';
  pathStr += a.x + diff.x / 3 + ',' + b.y + ' ';
  pathStr += b.x + ',' + b.y;
  
  return pathStr;
};

Node.prototype.connectTo = function(input){
  input.node = this;
  this.connected = true;
  this.domElement.classList.add('connected');
  
  input.domElement.classList.remove('empty');
  input.domElement.classList.add('filled');
  
  this.attachedPaths.push({
    input: input,
    path: input.path
  });
  
  var iPoint = input.getAttachPoint();
  var oPoint = this.getOutputPoint();
  
  var pathStr = this.createPath(iPoint, oPoint);
  
  input.path.setAttributeNS(null, 'd',pathStr);
};

Node.prototype.moveTo = function(point){
  this.domElement.style.top = point.y + 'px';
  this.domElement.style.left = point.x + 'px';
  this.updatePosition();
};

Node.prototype.initUI = function(){
  var that = this;
  
  // Make draggable
  $(this.domElement).draggable({
    containment: 'window',
    cancel: '.connection,.output',
    drag: function(event, ui){
      that.updatePosition();
    }
  });
  // Fix positioning
  this.domElement.style.position = 'absolute';
  
  document.body.appendChild(this.domElement);
  // Update Visual
  this.updatePosition();
};

// DEMO
// ========

// Node 1
var node = new Node('Another One');
node.addInput('Value1');
node.addInput('Value2');
node.addInput('Value3');

// Node 2
var node2 = new Node('Node 2');
node2.addInput('Text In');
node2.addInput('Value 5');

// Node 3
var node3 = new Node('Something Else');
node3.addInput('Color4');
node3.addInput('Position');
node3.addInput('Noise Octaves');

// Move to initial positions
node.moveTo({x: 150, y: 20});
node2.moveTo({x: 20, y: 70});
node3.moveTo({x:300, y:150});

// Connect Nodes
node.connectTo(node2.inputs[0]);
node3.connectTo(node2.inputs[1]);
node3.connectTo(node.inputs[0]);

// Add to DOM
node.initUI();
node2.initUI();
node3.initUI();