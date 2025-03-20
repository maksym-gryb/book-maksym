import { useState, useEffect } from 'react'
import useSWR from 'swr'
import { useCookies } from 'react-cookie'

interface CookieValues {
  session_id?: string;
}

export default function Login(){
    const [cookie, setCookie] = useCookies<'session_id', CookieValues>(['session_id']);
    const [isLoggedIn, setLoggedIn] = useState(false)

    // Check if session cookie is set on initial render
    useEffect(() => {
        console.log(cookie.session_id)
        if (cookie.session_id) {
            setLoggedIn(true)
        }
    }, [cookie])  // This will run only when cookies change

    function login(formData: FormData) {
        const username = formData.get("username")
        const password = formData.get("password")

        fetch("http://localhost:8000/login", {
            method: "POST",
            body: JSON.stringify({username: username, password: password}),
            credentials: 'include'
        })
        .then((res) => {
            if(!res.ok) return;

            setLoggedIn(true)
        })
    }

    if(isLoggedIn) return (
        <p>already logged in as ???</p>
    )

    return (
        <form action={login} className="flex">
        <input className="bg-gray-300" name="username" />
        <input className="bg-gray-300" name="password" type="password" />
        <button type="submit">login</button>
        </form>
    )
}