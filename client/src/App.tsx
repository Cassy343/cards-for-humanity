// import './App.css';

import { useEffect, useState } from "react";
import { Session, SessionContext } from "./Context";
import { WebsocketBuilder } from 'websocket-ts';

function App() {
    const [session, setSession] = useState<Session>({
        socket: null
    });

    useEffect(() => {
        new WebsocketBuilder(`ws://${window.location.host}/ws`)
            .onOpen((ws, e) => {
                setSession({
                    ...session,
                    socket: ws,
                });

                console.log("WS Connected");
            })
            .onClose((ws, e) => {
                setSession({
                    ...session,
                    socket: null
                });

                console.log("WS Disconnected");
            })
            .build();
    }, []);

    return (
        <SessionContext.Provider value={session}>
            <div className="App">
                <p>Hello World!</p>
            </div>
        </SessionContext.Provider>
    );
}

export default App;
