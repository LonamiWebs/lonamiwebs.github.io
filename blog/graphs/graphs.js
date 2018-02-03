// -------------------------------------------------------------------------- //
// Constants
var INTERPOLATION = 0.6;

var BOX_HEIGHT = 0.15; // %, from and to which drop the items
var BOX_HOVER = [255, 0, 0];
var BOX_NORMAL = [255, 200, 200];

var NODE_NORMAL = '#000';
var NODE_TEXT = '#fff';
var NODE_FONT = 'px Georgia';

var FRAME_DURATION = 25; // ms

// -------------------------------------------------------------------------- //
// Retrieve used HTML elements
canvas = document.getElementById('canvas');
matrixTable = document.getElementById('matrixTable');
matrixOrder = document.getElementById('matrixOrder');
matrixAccum = document.getElementById('matrixAccum');
matrixTableWrapper = document.getElementById('matrixTableWrapper');

// -------------------------------------------------------------------------- //
// Variables
var orderShown, accumShown;
c = canvas.getContext('2d');

// Box color, from which and to which nodes should be dragged
var boxColor = BOX_NORMAL.slice();

// Available nodes and their connection matrix- these should be kept in sync
var nodes = [];
var connMatrix = Array.matrix(0, 0, 0);

// Which node was the first we right clicked to create a new connection?
var startConn = null;

// Are we dragging any node yet?
var anyDragging = false;

// Highlight mode [row, col, color] for the matrix representing the graph
var highlight = [-1, -1, '#fff'];

function updateOrder() {
  orderShown = Number(matrixOrder.value);
  accumShown = matrixAccum.checked;
  calcShowConnMatrix();
}
updateOrder();

// -------------------------------------------------------------------------- //
// Declare a prototype (Node) to render the selectable elements
function Node(x, y, r, id) {
  this.x = x;
  this.y = y;
  this.r = r;
  this.id = id;
  this.color = null;
  this.dragging = false;

  this.draw = function() {
    if (this.dragging) {
      c.fillStyle = NODE_NORMAL;
      this.x = lerp(this.x, canvas.mouseX, INTERPOLATION);
      this.y = lerp(this.y, canvas.mouseY, INTERPOLATION);
    } else {
      c.fillStyle = this.color;
    }
    c.circle(this.x, this.y, this.r);

    c.fillStyle = NODE_TEXT;
    c.font = ''+r+NODE_FONT;
    // Arbitrary but nice
    var x = this.x - r * 0.3;
    var y = this.y + r * 0.3;

    c.fillText(""+this.id, x, y);
  }

  this.resetColor = function() {
    this.color = '#777';
  }
  this.resetColor();

  this.inBounds = function(x, y) {
    return Math.abs(this.x - x) < this.r &&
           Math.abs(this.y - y) < this.r;
  }
}

// -------------------------------------------------------------------------- //
// Node handling
function addNode(x, y) {
  nodes.push(new Node(x, y, 20, nodes.length + 1));
  limitOrder(nodes.length);

  var newMatrix = Array.matrix(nodes.length, nodes.length, 0);
  matrixCpy(connMatrix, newMatrix);
  setConnMatrix(newMatrix);
}

function deleteNode(at) {
  nodes.splice(at, 1);
  limitOrder(nodes.length);

  // Re-set the nodes IDs
  for (var i = 0; i != nodes.length; ++i) {
    nodes[i].id = i+1;
  }

  // Pop the at'th row (connections from the node we deleted)
  // and also the at'th column (connections to the node we deleted)
  setConnMatrix(matrixPop(connMatrix, at, at));
}

function clearNodes() {
  nodes.length = 0;
  limitOrder(1);
  resetConnections();
}

// -------------------------------------------------------------------------- //
// Connection matrix handling
function setConnMatrix(matrix) {
  connMatrix = matrix;
  makeNodesHighlightable(matrixTable, connMatrix);
  calcShowConnMatrix();
}

function resetConnections() {
  setConnMatrix(Array.matrix(nodes.length, nodes.length, 0));
}

// -------------------------------------------------------------------------- //
// Matrix representation
function calcShowConnMatrix() {
  if (orderShown > 1) {
    var multiplied = Array.matrix(nodes.length, nodes.length, 0);
    matrixCpy(connMatrix, multiplied);

    if (accumShown) {
      var shown = Array.matrix(nodes.length, nodes.length, 0);
      matrixCpy(connMatrix, shown);
    }

    for (var i = 1; i != orderShown; ++i) {
      multiplied = matrixMul(connMatrix, multiplied);
      if (accumShown) {
        matrixAdd(multiplied, shown);
      }
    }
    if (accumShown) {
      matrixRepr(shown, matrixTable, highlight);
    } else {
      matrixRepr(multiplied, matrixTable, highlight);
    }
  } else {
    matrixRepr(connMatrix, matrixTable, highlight);
  }
}

// Makes a table "highlightable" (when the mouse hovers,
// it will be redrawn) by using the specified matrix.
function makeNodesHighlightable(table, matrix) {
  table.matrix = matrix;
  table.addEventListener('mousemove', function(e) {
    if (!table.matrix && !table.matrix[0])
      return;

    // TODO I know, this shouldn't have to go here and I should make a
    //      better script to show the whole path and stuff... oh well.
    if (orderShown > 1) {
      if (highlight[0] != -1 || highlight[1] != -1) {
        highlight[0] = highlight[1] = -1;
        for (var i = 0; i != nodes.length; ++i)
          nodes[i].resetColor();
        calcShowConnMatrix();
      }
      return;
    }

    var rect = table.getBoundingClientRect();
    var x = e.clientX - rect.left;
    var y = e.clientY - rect.top;

    var colWidth = rect.width / (table.matrix[0].length+1);
    var col = Math.floor(x / colWidth)-1;
    
    var rowHeight = rect.height / (table.matrix.length+1);
    var row = Math.floor(y / rowHeight)-1;

    if (row != highlight[0] || col != highlight[1]) {
      highlight[0] = row;
      highlight[1] = col;
      if (row >= 0 && col >= 0) {
        if (table.matrix[row][col]) {
          // there is a connection so color them
          nodes[row].color = '#77f';
          nodes[col].color = '#7f7';
          highlight[2] = '#afa';
        } else {
          // no connection so reset its colors
          for (var i = 0; i != nodes.length; ++i)
            nodes[i].resetColor();

          highlight[2] = '#faa';
        }
      }
      calcShowConnMatrix();
    }

  }, false);
}

// -------------------------------------------------------------------------- //
// Canvas rendering
function renderElements() {
  c.clear();
  if (canvas.mouseDown && canvas.mouseY < canvas.height * BOX_HEIGHT) {
    boxColor = lerpArray(boxColor, BOX_HOVER, 0.2);
  } else {
    boxColor = lerpArray(boxColor, BOX_NORMAL, 0.2);
  }
  c.rect(0, 0, canvas.width, canvas.height * BOX_HEIGHT);
  c.fillStyle = 'rgb(' +
                  Math.round(boxColor[0]) + ',' +
                  Math.round(boxColor[1]) + ',' +
                  Math.round(boxColor[2]) + ')';
  c.fill();
  
  
  if (nodes.length == 0)
    return;

  for (var i = 0; i != nodes.length; ++i)
    nodes[i].draw();

  c.strokeStyle = '#f00';

  var ii = connMatrix.length;
  var jj = connMatrix.length;

  var x0, y0, x1, y1, cos, sin;
  var rsq = Math.pow(nodes[0].r + 40, 2); // Radius SQuared + some margin
  for (var i = 0; i != ii; ++i) {
    for (var j = 0; j != jj; ++j) {
      if (connMatrix[i][j]) {
        // Non-zero item, this implies a connection between nodes.
        // An element on the matrix connects row (i) -> column (j)
        x0 = nodes[i].x;
        y0 = nodes[i].y;
        x1 = nodes[j].x;
        y1 = nodes[j].y;

        // Interpolate a bit to exit the radius unless we're too close
        if (Math.pow(y1 - y0, 2) + Math.pow(x1 - x0, 2) > rsq) {
          var angle = Math.atan2(y1 - y0, x1 - x0);
          cos = Math.cos(angle) * nodes[0].r;
          sin = Math.sin(angle) * nodes[0].r;
          x0 += cos;
          y0 += sin;
          x1 -= cos;
          y1 -= sin;
        }
        c.arrow(x0, y0, x1, y1);
      }
    }
  }

  if (startConn != null) {
    c.arrow(nodes[startConn].x, nodes[startConn].y,
            canvas.mouseX, canvas.mouseY);
  }
}

function renderLoop() {
  renderElements();
  setTimeout(renderLoop, FRAME_DURATION);
}

// -------------------------------------------------------------------------- //
// HTML interaction

// To limit the maximum orden which can be specified on the HTML and
// also clamp the currently shown order to the limit if it's exceeded
function limitOrder(value) {
  matrixOrder.max = value;
  if (orderShown > value) {
    orderShown = value;
    matrixOrder.value = '' + value;
  }
}

canvas.addEventListener('mousedown', function(e) {
  if (e.button == 0) {
      if (canvas.mouseY < canvas.height * BOX_HEIGHT) {
        // Add node
        addNode(canvas.mouseX, 0);
        nodes[nodes.length - 1].dragging = true;
        anyDragging = true;
      } else {
        // Drag nodes
        for (var i = nodes.length; i--;) {
          if (nodes[i].inBounds(canvas.mouseX, canvas.mouseY)) {
            nodes[i].dragging = true;
            anyDragging = true;
            break;
          }
        }
    }
    canvas.style.cursor = anyDragging ? "crosshair" : "default";

  } else if (e.button == 2) {
    // Right button, select two nodes to join
    if (startConn == null) {
      for (var i = nodes.length; i--;) {
        if (nodes[i].inBounds(canvas.mouseX, canvas.mouseY)) {
          startConn = i;
          break;
        }
      }
    }
  } else if (e.button == 1) {
    // Middle click, clear the connections from and to a given node
    for (var i = nodes.length; i--;) {
      if (nodes[i].inBounds(canvas.mouseX, canvas.mouseY)) {
        for (var j = 0; j != nodes.length; ++j)
          connMatrix[i][j] = 0;
        break;
      }
    }
  }

  canvas.mouseDown = e.button == 0 ? 'L' : 'R';
}, false);

canvas.addEventListener('mouseup', function(e) {
  canvas.mouseDown = '';

  if (anyDragging) {
    anyDragging = false;
    for (var i = nodes.length; i--; ) {
      if (nodes[i].dragging) {
        if (canvas.mouseY < canvas.height * BOX_HEIGHT) {
          deleteNode(i);
        } else {
          nodes[i].dragging = false;
        }
      }
    }
  } else if (startConn != null) {
    for (var i = nodes.length; i--;) {
      if (nodes[i].inBounds(canvas.mouseX, canvas.mouseY)) {
        // Node row connected to node column
        if (i != startConn)
          connMatrix[startConn][i] = 1;
        break;
      }
    }
    startConn = null;
  }

  calcShowConnMatrix();
}, false);

canvas.addEventListener('mousemove', function(e) {
  var rect = canvas.getBoundingClientRect();
  canvas.mouseX = e.clientX - rect.left;
  canvas.mouseY = e.clientY - rect.top;

}, false);

// -------------------------------------------------------------------------- //
// Represent the example we walked through
function lilNoise() {
  return (Math.random() - 0.5) * canvas.width * 0.05;
}

function getNoiseX(relative) {
  return canvas.width * relative + lilNoise();
}

function getNoiseY(relative) {
  return canvas.height * relative + lilNoise();
}

addNode(getNoiseX(0.2), getNoiseY(0.5));
addNode(getNoiseX(0.5), getNoiseY(0.3));
addNode(getNoiseX(0.8), getNoiseY(0.5));
addNode(getNoiseX(0.7), getNoiseY(0.7));
addNode(getNoiseX(0.3), getNoiseY(0.7));

setConnMatrix([
  [0, 1, 0, 0, 0],
  [1, 0, 0, 0, 1],
  [0, 0, 0, 1, 0],
  [0, 1, 1, 0, 0],
  [1, 0, 0, 1, 0]
]);

calcShowConnMatrix();


// Let's go!
renderLoop();
