// Waiting for declarative-custom-elements ;w;
// DSD has SSR benefits but makes more HTML duplication 
// than I am comfortable with.
//
// Recommend using html/css tagged template literals
// (eg) html`foo`, css`bar` with an extension for highlighting
// VSCODE: `FAST Tagged Template Literals`
// Dunno for any other IDEs yet, sorry.
//
// Yes, this is basically a light reimpl of Lit.
class NonDeclarativeElement extends HTMLElement {
    html = "";
    css = "";
    static name;
    static attrs = [];

    constructor() {
        super();
    }

    start(shadow) {
        // ...
    }

    connectedCallback() {
        const shadow = this.attachShadow({ mode: "closed" });
        shadow.innerHTML = `<style>
            ${this.css.trim()}
        </style>
        ${this.html.trim()}`;

        this.shadow = shadow;

        this.start(shadow);
    }



    static get observedAttributes() {
        return this.attrs;
    }
    attributeChangedCallback(property, oldValue, newValue) {
        if (oldValue === newValue) return;
        this[property] = newValue;
    }

    static define() {
        if (this.name === undefined) {
            console.warn("No template!");
        }
        customElements.define(this.name, this);
    }
}

const html = (strings, ...values) => String.raw({ raw: strings }, ...values);
const css = (strings, ...values) => String.raw({ raw: strings }, ...values);


class DeclarativeElement extends HTMLElement {
    static element_name = "never";

    static attrs = [];
    dryAttributes = [];

    constructor() {
        super();
    }

    connectedCallback() {
        const supportsDeclarative = HTMLElement.prototype.hasOwnProperty("attachInternals");
        const internals = supportsDeclarative ? this.attachInternals() : undefined;

        // check for a Declarative Shadow Root.
        let shadow = internals?.shadowRoot;

        if (!shadow) {
            // there wasn't one. create a new Shadow Root:
            shadow = this.attachShadow({
                mode: "closed",
            });
            const templateEl = document.querySelector(`template[element="${this.constructor.element_name}"]`);
            shadow.append(templateEl.content.cloneNode(true));
        }

        this.shadow = shadow;

        this.dryAttributes.forEach(element => {
            this.attributeSet(element.property, element.oldValue, element.newValue);
        });
        this.start(shadow);
    }

    static get observedAttributes() {
        return this.attrs;
    }
    attributeChangedCallback(property, oldValue, newValue) {
        if (oldValue === newValue) return;
        this[property] = newValue;

        if (this.shadow == null) {
            this.dryAttributes.push({ property: property, oldValue: oldValue, newValue: newValue });
        } else {
            this.attributeSet(property, oldValue, newValue);
        }
    }

    static define() {
        if (this.element_name === undefined) {
            console.warn("No template!");
        }
        customElements.define(this.element_name, this);
    }

    start(shadow) {
        // ...
    }

    attributeSet(property, oldValue, newValue) {
        // ...
    }
}


export { NonDeclarativeElement, html, css, DeclarativeElement };