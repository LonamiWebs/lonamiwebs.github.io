// Custom tiny parser for csv tables. They're more comfortable.
var tables = document.getElementsByClassName('csv');
for (var i = 0; i != tables.length; ++i) {
  var lines = tables[i].innerHTML.split('\n');
  var result = ['<table>'];
  var line = null;
  for (var j = 0; j != lines.length; ++j) {
    line = lines[j].trim();
    if (line) {
      result.push('<tr><td>');
      result.push(lines[j].replace(/\s*;\s*/g, '</td><td>'));
      result.push('</td></tr>');
    }
  }
  result.push('</table>');
  tables[i].innerHTML = result.join('');
}

// Custom tiny math parser. I like writing parsers.
// 'x^2n ' becomes 'x<sup>2n</sup> '
// 'x_5n ' becomes 'x<sub>5n</sub> '
// 'a * b' becomes 'a × b'
// ' '     becomes '&nbsp;'
function mathify(element) {
  var c = null;
  var closeOnSpace = null;
  var content = element.innerHTML.split('');
  for (var j = 0; j != content.length; ++j) {
    c = content[j];
    if (c == '*') {
      content[j] = '×';
    } else if (c == '^') {
      content[j] = '<sup>';
      closeOnSpace = '</sup>';
    } else if (c == '_') {
      content[j] = '<sub>';
      closeOnSpace = '</sub>';
    } else if (c == ' ') {
      if (closeOnSpace == null) {
        content[j] = '&nbsp;';
      } else {
        content[j] = closeOnSpace + ' ';
        closeOnSpace = null;
      }
    } else if (c == '\n' && closeOnSpace != null) {
        content[j] = closeOnSpace + '\n';
        closeOnSpace = null;
    }
  }
  if (closeOnSpace != null) {
    content.push(closeOnSpace.trim());
  }

  element.innerHTML = content.join('');
}

var maths = document.getElementsByTagName('mark');
for (var i = 0; i != maths.length; ++i)
  mathify(maths[i]);

var maths = document.getElementsByClassName('math');
for (var i = 0; i != maths.length; ++i)
  mathify(maths[i]);
