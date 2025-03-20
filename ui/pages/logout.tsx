import { useCookies } from 'react-cookie'

interface CookieValues {
  session_id?: string;
}

export default function logout() {
    const [cookie, setCookie, removeCookie] = useCookies<'session_id', CookieValues>(['session_id']);

    removeCookie("session_id");

    return (
        <div>
            you have been logged-out
        </div>
    )
}