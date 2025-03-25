import { useState } from 'react';
import { useCookies } from 'react-cookie'

interface CookieValues {
  session_id?: string;
}

function logoutAction(setState: Function, removeCookie: Function){
    fetch("http://localhost:8000/logout", {
        credentials: 'include'
    })
    .then((res) => {
        if(!res.ok) {
            return;
        }

        removeCookie("session_id");
        setState(true)
    })

}

export default function logout() {
    const [cookie, setCookie, removeCookie] = useCookies<'session_id', CookieValues>(['session_id']);
    const [state, setState] = useState(false);

    if(state)
    {
        return (
            <div>Logged out!</div>
        );
    }

    logoutAction(setState, removeCookie);
    return (
        <div>
            logging you out...     
        </div>
    )
}