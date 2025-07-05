async function new_user() {
    const tz = Intl.DateTimeFormat().resolvedOptions().timeZone;

    await fetch(`${location}user/new`, {
        method: "POST",
        headers: {
            "Content-Type": "application/json",
        },
        body: JSON.stringify({ timezone: tz }),
    });
}

async function check_invalidate() {
    const invalidate_cookie = await cookieStore.get("invalidate");
    if (invalidate_cookie?.value === "true") {
        await new_user();
        await cookieStore.delete("invalidate");
        htmx.trigger("body", "login", {});
    }
}

await check_invalidate();