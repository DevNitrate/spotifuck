let username = document.getElementById("username")
let pswd = document.getElementById("pswd")
let error = document.getElementById("error")
let login = document.getElementById("login")
let sub = document.getElementById("sub")

login.addEventListener("submit", async (e) => {
    e.preventDefault()

    sub.innerText = "..."
    
    let req_url = "/auth/login/" + username.value + "/" + pswd.value
    let req = await fetch(req_url, { method: "POST" })
    let res = await req.json()
    sub.innerText = "se connecter"
    
    if (res.error == "invalid username") {
        error.innerText = "ce nom d'utilisateur n'existe pas"
    } else if (res.error == "invalid password") {
        sub.innerText = "se connecter"
        error.innerText = "mot de passe incorrect"
    } else if (res.error == "none") {
        window.location.href = "/"
    }
})