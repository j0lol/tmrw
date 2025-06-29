async function new_user() {
  const tz = Intl.DateTimeFormat().resolvedOptions().timeZone;
  console.log(tz);

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
      console.log("account created!");
      Cookies.remove("invalidate");
      htmx.trigger("body", "login", {});
    });
  }
}

check_invalidate();
