// import { useState, useEffect } from "react"
import useSWR from 'swr'

interface User {
    id: number,
    username: string,
    role: string
}

const fetcher = (...args: [RequestInfo | URL, RequestInit?]) => fetch(args[0], {...args[1], credentials: 'include'}).then((res) => res.json())

export default function Page(){
    const {data, error} = useSWR<User>("http://localhost:8000/profile", fetcher)

    if (!data) return <p>Loading data...</p>
    if (error) return <p>error</p>

    return (
        <>
            <h1>Profile</h1>
            <ul>
                <li>Id: {data.id}</li>
                <li>Username: {data.username}</li>
                <li>Role: {data.role}</li>
            </ul>
        </>
    )
}