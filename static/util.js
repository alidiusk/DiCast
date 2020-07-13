"use strict";

window.MutationObserver = window.MutationObserver
    || window.WebKitMutationObserver
    || window.MozMutationObserver;

const select = document.getElementById("dice-select");

const observer = new MutationObserver(function(mutation, observer) {
  select.value = select.getAttribute("value");
});

const config = {
    attributes: true // this is to watch for attribute changes.
};

observer.observe(select, config);
