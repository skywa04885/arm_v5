import { Event, UnlistenFn, listen } from "@tauri-apps/api/event";
import React from "react";

export function useListen<Payload>(event: string, callback: (payload: Payload) => any) {
    React.useEffect((): () => void => {
        const unlisten: Promise<UnlistenFn> = listen(event,
            (event: Event<Payload>): void => {
                callback(event.payload);
            });

        return (): void => {
            unlisten.then(f => f());
        };
    }, [event, callback]);
}
