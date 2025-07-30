let username = document.getElementById("username")
let pswd = document.getElementById("pswd")
let error = document.getElementById("error")
let create = document.getElementById("create")
let sub = document.getElementById("sub")

create.addEventListener("submit", async (e) => {
    e.preventDefault()

    sub.innerText = "..."

    let req_url = "/auth/create/" + username.value + "/" + pswd.value
    let req = await fetch(req_url, { method: "POST" })
    let res = await req.json()

    if (res.exists == false) {
        window.location.href = "/login"
    } else {
        error.innerText = "ce nom d'utilisateur est déjà utilisé"
        sub.innerText = "s'inscrire"
    }
})