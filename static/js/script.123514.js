"use strict";

async function new_user() {
  const tz = Intl.DateTimeFormat().resolvedOptions().timeZone;

  const response = await fetch(`${location}user/new`, {
    method: "POST",
    headers: {
      "Content-Type": "application/json",
    },
    body: JSON.stringify({ timezone: tz }),
  });
}

function check_invalidate() {
  if (Cookies.get("invalidate") === "true") {
    new_user().then(() => {
      Cookies.remove("invalidate");
      htmx.trigger("body", "login", {});
    });
  }
}

check_invalidate();


// Credit: https://css-tricks.com/auto-growing-inputs-textareas/#comment-1787000
// Please use theirs, not mine. I have modified it for my own purposes.
const autoWidthTemplate = document.createElement('template');
autoWidthTemplate.innerHTML = `
  <style>
    :host {
      display: inline-block;
      vertical-align: top;
      align-items: center;
      position: relative;
    }
    :host::after {
      width: auto;
      min-width: 1em;
      content: attr(data-value) ' ';
      visibility: hidden;
      white-space: pre-wrap;
      font: inherit;
      text-wrap: nowrap;
      pointer-events: none;
      touch-action: none; /* these two rules are just in case*/ 
      z-index: -99; /* prevents inputs */
    }
    input {
      font: inherit;
      width: 100%;
      min-width: 6ch;
      position: absolute;
      border: 0;
      padding: 0; /* for iOS */
      background: none;
    }
  </style>
  <input size='1'></input>
`;
class AutoWidthInput extends HTMLElement {
  static get formAssociated() {
    return true;
  }

  constructor() {
    super();
    this.internals = this.attachInternals();

    this._shadowRoot = this.attachShadow({ mode: 'open' });
    this._shadowRoot.appendChild(autoWidthTemplate.content.cloneNode(true));
    this._input = this._shadowRoot.querySelector('input');

    this.value_ = this._input.value;

  }
  _manageValidity() {
    const value = this._input.value;
    const required = this._input.hasAttribute("required");

    if (value === '' && required) {
      this.internals.setValidity({
        valueMissing: true
      }, 'This field is required', this._input);
    } else {
      this.internals.setValidity({});
    }
  }

  connectedCallback() {
    this.value = this._input.value;

    this._input.addEventListener('input', (e) => {
      this.value = this._input.value;
    });
  }

  formResetCallback() {
    this.value = "";
  }
  get value() {
    return this._input.value;
  }
  set value(v) {
    this._input.value = v;
    this.dataset.value = v;
    this.internals.setFormValue(v);
    this._manageValidity();
  }
  static get observedAttributes() {
    return ['placeholder', 'name', 'required'];
  }
  attributeChangedCallback(name, oldVal, newVal) {
    this._input.setAttribute(name, newVal);
  }
}
window.customElements.define('autowidth-input', AutoWidthInput);
