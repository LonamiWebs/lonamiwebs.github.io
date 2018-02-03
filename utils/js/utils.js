Array.prototype.contains = function(value) {
    return this.indexOf(value) > -1;
}

function fadeIn(elementId) {
    var element = document.getElementById(elementId);
    // Avoid single JavaScript rounds (http://stackoverflow.com/q/24148403)
    element.offsetWidth = element.offsetWidth;
    element.classList.add("fadein");
}
