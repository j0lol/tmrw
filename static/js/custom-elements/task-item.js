import { DeclarativeElement } from "../modules/component-element.js";

const tomorrow_ul = document.querySelector(".card-body.tomorrow ul");


class TaskItem extends DeclarativeElement {
    static element_name = "task-item";

    static attrs = ["task_id", "checked", "day"];

    start(shadow) {
        const messageSpan = shadow.querySelector("slot.message");
        this.message = messageSpan.textContent;

        messageSpan.addEventListener("click", async () => {
            messageSpan.classList.add("is-loading");

            const response = await fetch("/task/complete", {
                method: "POST",
                body: new URLSearchParams({
                    "id": this.task_id
                })
            });

            messageSpan.classList.remove("is-loading");

            if (response.ok) {
                this.toggleAttribute("checked");
            }
        });

        const btn_delete = shadow.querySelector("button.button-delete");

        btn_delete.addEventListener("click", async () => {
            btn_delete.classList.add("is-loading");

            const response = await fetch("/task/delete", {
                method: "POST",
                body: new URLSearchParams({
                    "id": this.task_id
                })
            });

            btn_delete.classList.remove("is-loading");

            if (response.ok) {
                this.remove();
            }
        })

        const btn_pushback = shadow.querySelector("button.button-pushback");

        btn_pushback.addEventListener("click", async () => {
            btn_pushback.classList.add("is-loading");

            const response = await fetch("/task/pushback", {
                method: "POST",
                body: new URLSearchParams({
                    "id": this.task_id
                })
            });

            btn_pushback.classList.remove("is-loading");

            if (response.ok) {
                tomorrow_ul.appendChild(this);
                shadow.querySelectorAll("details").forEach(element => {
                    element.removeAttribute("open");
                });
            }
        })

    }

    attributeSet(property, oldValue, newValue) {
        if (newValue == null) {
            this.shadow.querySelector(".list-item").removeAttribute(property);

        } else {
            this.shadow.querySelector(".list-item").setAttribute(property, newValue);

        }
    }


}

TaskItem.define();


export { TaskItem }