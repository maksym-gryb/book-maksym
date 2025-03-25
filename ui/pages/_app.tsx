import "@/styles/globals.css";
import type { AppProps } from "next/app";
import { CookiesProvider } from 'react-cookie';
import { ToastContainer } from 'react-toastify';


export default function App({ Component, pageProps }: AppProps) {
  return (
    <CookiesProvider defaultSetOptions={{ path: '/' }}>
      <Component {...pageProps} />
      <ToastContainer
        // position="top-right"
        autoClose={2000}
        hideProgressBar={true}
        newestOnTop={false}
        closeOnClick={true}
        rtl={false}
        pauseOnFocusLoss
        draggable
        pauseOnHover
        theme="dark"
        // transition={Bounce}
      />
    </CookiesProvider>
  )
}
