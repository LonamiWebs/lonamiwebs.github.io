// Enhancing prototypes
CanvasRenderingContext2D.prototype.clear = function() {
  // http://stackoverflow.com/a/9722502/4759433
  this.clearRect(0, 0, this.canvas.width, this.canvas.height);
  this.beginPath(); // Clear strokes
};

var TWOPI = 2 * Math.PI;
CanvasRenderingContext2D.prototype.circle = function(x, y, r) {
  this.beginPath();
  this.arc(x, y, r, 0, TWOPI);
  this.fill();
  this.beginPath();
};

var SIXTHPI = Math.PI / 6.0;
CanvasRenderingContext2D.prototype.arrow = function(
    x0, y0, x1, y1, headLen=10) {
  // http://stackoverflow.com/a/6333775
  var angle = Math.atan2(y1 - y0, x1 - x0);
  this.moveTo(x0, y0);

  this.lineTo(x1, y1);
  this.lineTo(x1 - headLen * Math.cos(angle - SIXTHPI),
              y1 - headLen * Math.sin(angle - SIXTHPI));

  this.moveTo(x1, y1);
  this.lineTo(x1 - headLen * Math.cos(angle + SIXTHPI),
              y1 - headLen * Math.sin(angle + SIXTHPI));

  this.stroke();
}

String.prototype.format = function() {
  // http://stackoverflow.com/a/4673436/4759433
  var args = arguments;
  return this.replace(/{(\d+)}/g, function(match, number) {
    return typeof args[number] != 'undefined' ? args[number] : match;
  });
};

String.prototype.lpad = function(pad) {
  return String(pad + this).slice(-pad.length);
};

Array.matrix = function(numrows, numcols, initial) {
  // http://stackoverflow.com/a/30056867/4759433
  var arr = [];
  for (var i = 0; i != numrows; ++i) {
    var columns = [];
    for (var j = 0; j != numcols; ++j)
      columns[j] = initial;
    arr[i] = columns;
  }
  return arr;
}

function matrixCpy(src, dst) {
  // Ensure it has at least one item
  if (src.length != 0 && dst.length != 0) {
    var ii = Math.min(src.length,    dst.length);
    var jj = Math.min(src[0].length, dst[0].length);

    for (var i = 0; i != ii; ++i)
      for (var j = 0; j != jj; ++j)
        dst[i][j] = src[i][j];
  }
}

function matrixAdd(src, dst) {
  // Ensure it has at least one item
  if (src.length != 0 && dst.length != 0) {
    var ii = Math.min(src.length,    dst.length);
    var jj = Math.min(src[0].length, dst[0].length);

    for (var i = 0; i != ii; ++i)
      for (var j = 0; j != jj; ++j)
        dst[i][j] += src[i][j];
  }
}

function matrixMul(a, b) {
  var ii = a.length;
  var jj = b[0].length;
  var c = Array.matrix(ii, jj, 0);

  var kk = a[0].length;
  if (kk == b.length) // Ensure nxp times pxm
    for (var i = 0; i != ii; ++i)
      for (var j = 0; j != jj; ++j)
        for (var k = 0; k != kk; ++k)
          c[i][j] += a[i][k] * b[k][j];

  return c;
}

// Pops the i'th row and j'th column off the matrix
function matrixPop(matrix, i, j) {
  if (!matrix || !matrix[0])
    return Array.matrix(0, 0, 0);

  var ii = matrix.length;
  var jj = matrix[0].length;
  if (i < 0 || i >= ii || j < 0 || j >= jj)
    return matrix;

  var srci, srcj, dsti, dstj;
  var result = Array.matrix(ii-1, jj-1, 0);

  dsti = 0;
  for (srci = 0; srci < ii; ++srci) {
    dstj = 0;
    if (srci != i) {
      for (srcj = 0; srcj < jj; ++srcj) {
        if (srcj != j) {
          result[dsti][dstj] = matrix[srci][srcj];
          ++dstj;
        }
      }
      ++dsti;
    }
  }

  return result;
}

function matrixStr(a) {
  if (!a || !a[0])
    return '[]';

  var ii = a.length;
  var jj = a[0].length;
  var jjm1 = jj - 1;

  var max = a[0][0];
  for (var i = 0; i != ii; ++i)
    for (var j = 0; j != jj; ++j)
      if (a[i][j] > max)
        max = a[i][j];

  var result = '';
  max = '' + max;
  var pad = '';
  for (var i = 0; i != max; ++i)
    pad += ' ';

  for (var i = 0; i != ii; ++i) {
    result += '[';
    for (var j = 0; j != jjm1; ++j) {
      result += ('' + a[i][j]).lpad(pad);
      result += ',';
    }
    result += ('' + a[i][jjm1]).lpad(pad);
    result += ']\n';
  }

  return result;
}

// highlight = [belowRow, belowCol, color]
function matrixRepr(matrix, table, highlight=null) {
  if (!matrix || !matrix[0]) {
    table.innerHTML = '';
    return;
  }

  if (!highlight)
    highlight = [-1, -1, '#000'];

  var result = '';
  var ii = matrix.length;
  var jj = matrix[0].length;

  result += '<tr><td></td>';
  for (var j = 0; j != jj; ++j)
    result += '<td><b>' + (j+1) + '</b></td>';
  result += '</tr>';

  for (var i = 0; i != ii; ++i) {
    result += '<tr><td><b>' + (i+1) + '</b></td>';
    for (var j = 0; j != jj; ++j) {
      // We want to higlight 0..col on row and 0..row on col
      if ((i <= highlight[0] && j == highlight[1]) ||
          (i == highlight[0] && j <= highlight[1])) {
        result += '<td style="background-color:' + highlight[2] +
                  ';">' + matrix[i][j] + '</td>';
      } else {
        result += '<td>' + matrix[i][j] + '</td>';
      }
    }
    result += '</tr>';
  }

  table.innerHTML = result;
}

function lerp(start, end, t) {
  return start + t * (end - start);
}

function lerpArray(start, end, t) {
  var result = [];
  var ii = Math.min(start.length, end.length);
  for (var i = 0; i != ii; ++i)
    result.push(start[i] + t * (end[i] - start[i]));
  return result;
}
