// To be paired with a server-side rendering step.
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


export { DeclarativeElement };