const mathify = content => {
  let result = ''
  let close = null;
  let symbol = null;
  Array.from(content).forEach(c => {
      if (symbol !== null) {
          if (c === ' ') {
              if (symbol === 'rightarrow') {
                  result += '→'
              } else if (symbol === 'geq') {
                  result += '≥'
              } else {
                  result += '?'
              }
              symbol = null;
              result += '&nbsp;'
          } else {
              symbol += c
          }
      } else if (c == '$') {
          symbol = ''
      } else if (c === '*') {
          result += '×'
      } else if (c === '^') {
          result += '<sup>'
          close = '</sup>'
      } else if (c === '_') {
          result += '<sub>';
          close = '</sub>';
      } else if (c === '·') {
        result += '&nbsp;'
      } else if (c === '#') {
          result += '<strong>';
          close = '</strong>';
      } else if (c === ' ' || c == '\n') {
          if (close !== null) {
              result += close
              close = null
          }
          result += '&nbsp;'
      } else {
          result += c
      }
  })
  if (close !== null) {
      result += close;
  }
  return `<em class="math">${result.trim()}</em>`
}

const parse_maths = html => html.replaceAll(/\(\(.+?\)\)/g, match =>
  mathify(match.slice(2, -2))
);

Array.from(document.getElementsByClassName('matrix')).forEach(matrix => {
  let result = '<table>'
  matrix.innerHTML.trim().split('\\\\').forEach(row => {
      result += '<tr>'
      row.trim().split("'").forEach(elem => {
          result += `<td>${mathify(elem.trim())}</td>`
      })
      result += '</tr>'
  })
  matrix.innerHTML = result
})

Array.from(document.getElementsByTagName('p')).forEach(paragraph => {
  paragraph.innerHTML = parse_maths(paragraph.innerHTML)
})
