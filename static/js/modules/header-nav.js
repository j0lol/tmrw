const btn_today = document.querySelector("button.header-today");
const btn_tomorrow = document.querySelector("button.header-tomorrow");
const main = document.querySelector("#main");
const inp_when = document.querySelector("#input-when");

btn_today.addEventListener("click", toggle_today);
btn_tomorrow.addEventListener("click", toggle_tomorrow);

async function toggle_today() {
    btn_today.setAttribute("active", "");
    btn_tomorrow.removeAttribute("active");

    main.setAttribute("day", "today");
    inp_when.setAttribute("value", "today");
}
async function toggle_tomorrow() {
    btn_tomorrow.setAttribute("active", "");
    btn_today.removeAttribute("active");

    main.setAttribute("day", "tomorrow");
    inp_when.setAttribute("value", "tomorrow");
}