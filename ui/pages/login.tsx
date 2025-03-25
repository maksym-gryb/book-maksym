import { useState, useEffect } from 'react'
import { useCookies } from 'react-cookie'
import { toast } from 'react-toastify';


interface CookieValues {
  session_id?: string;
}

const contextClass = {
  success: "bg-blue-600",
  error: "bg-red-600",
  info: "bg-gray-600",
  warning: "bg-orange-400",
  default: "bg-indigo-600",
  dark: "bg-white-600 font-gray-300",
};

export default function Login(){
    const [cookie, setCookie] = useCookies<'session_id', CookieValues>(['session_id']);
    const [isLoggedIn, setLoggedIn] = useState(false)
    // const notify = (e: string) => toast<string>(e, {className: "bg-red-600"});

    // Check if session cookie is set on initial render
    useEffect(() => {
        console.log(cookie.session_id)
        if (cookie.session_id) {
            setLoggedIn(true)
        }
    }, [cookie])  // This will run only when cookies change

    function login(formData: FormData) {
        const username = formData.get("username") as string
        const password = formData.get("password") as string

        const formBody = new URLSearchParams();
        formBody.append("username", username);
        formBody.append("password", password);

        fetch("http://localhost:8000/login", {
            method: "POST",
            headers: {
                "Content-Type": "application/x-www-form-urlencoded",
            },
            // body: JSON.stringify({username: username, password: password}),
            body: formBody.toString(),
            credentials: 'include'
        })
        .then((res) => {
            if(!res.ok) {
                toast.error("Failed to login!", {
                    position: "top-center"
                });
                // notify("Failed to login");
                return;
            }

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