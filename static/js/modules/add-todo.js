const form_add = document.querySelector("form.addtodo");
const list_today = document.querySelector("article div.card-body.today ul");
const list_tomorrow = document.querySelector("article div.card-body.tomorrow ul");

form_add.addEventListener("submit", async (event) => {
    event.preventDefault();
    form_add.classList.add("is-loading");

    const body = new URLSearchParams(new FormData(form_add));

    const response = await fetch("/task/new", {
        method: "POST",
        body: new URLSearchParams(new FormData(form_add))
    });

    form_add.classList.remove("is-loading");

    if (response.ok) {
        form_add.reset();

        console.log("Response OK");

        let task = JSON.parse(await response.json());
        console.log(JSON.stringify(task));
        console.log(task.text);

        const when = body.get("when");

        let list;
        if (when === "today") {
            list = list_today;
        } else if (when === "tomorrow") {
            list = list_tomorrow;
        }

        let taskItem = document.createElement("task-item");
        taskItem.setAttribute("task_id", task.id);
        taskItem.setAttribute("day", when);
        if (task.checked) taskItem.setAttribute("checked", "");
        taskItem.innerHTML = `<span slot="message">${task.text}</span>`;

        list.appendChild(taskItem);
    }

})