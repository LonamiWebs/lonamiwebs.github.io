// Variables
hashtags = [];

// Adding and removing
function addHashtag() {
    var input = document.getElementById('hashtagInput');
    var value = input.value.trim().split(' ')[0].toLowerCase();
    if (!value || hashtags.contains(value))
        return;

    // Clear value and push new hashtag
    input.value = '';
    var id = 'hashtag_'+hashtags.length;
    hashtags.push(value);

    // Show new hashtag
    var newElem = '<li id="'+id+'" class="fadable" onClick="';
    newElem += "clearHashtag('"+id+"');";
    newElem += '">'+value+'</li>';

    document.getElementById('filters').innerHTML += newElem;
    fadeIn(id);
    filterByHashtag();
}

function clearHashtag(tag) {
    document.getElementById(tag).outerHTML = '';
    hashtags.pop(tag);
    filterByHashtag();
}

function clearHashtags() {
    hashtags.length = 0;
    document.getElementById('filters').innerHTML = '';
    filterByHashtag();
}

// Filtering
function filterByHashtag() {
    var cards = document.getElementById('allcards').children;
    for (var i = 0; i < cards.length; ++i) {
        var card = cards[i];
        var filters = card.getElementsByClassName('filterlist')[0].children;
        
        // Determine if this card has the correct filters (hashtags)
        var found = 0;
        for (var j = 0; j < filters.length; ++j) {
            filter = filters[j].innerHTML;
            if (hashtags.contains(filter))
                ++found;
        }
        
        // Set its visibility based on whether all hashtags were found
        var ok = found == hashtags.length;
        card.style.display = ok ? '' : 'none';
    }
}

// Events
document.getElementById('hashtagInput').onkeypress = function(e) {
    if (e.which == 13) {
        addHashtag();
        return false;
    }

    return (e.which >= 65 && e.which < 90) ||
           (e.which >= 97 && e.which < 122);
}

