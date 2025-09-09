import { useCallback, useEffect, useState, type FunctionComponent, type PropsWithChildren } from "react";
import { client } from "../api";
import { RetryErrorBoundary } from "./RetryBoundary";
import { useLoaded } from "../hook/useLoaded";

const STORE_DATA_KEY = "data-cache";
const STORE_TOKEN_DATA_KEY = "token-data-cache";


const checkLoadingToken = () => {

    const data = localStorage.getItem(STORE_TOKEN_DATA_KEY);

    if (data === null) {
        return false;
    }

    client.defaults.headers.common.Authorization = `Bearer ${data}`;
    return true;
}

const RequestSession: FunctionComponent<PropsWithChildren> = ({
    children
}) => {
    const result =  useLoaded(useCallback(async (): Promise<{ loginUrl: string, doneUrl: string }> => {
        const data = localStorage.getItem(STORE_DATA_KEY);
        if (data) return JSON.parse(data);
        try {
            return await client.post("/-/v1/login").then((result) => {
                localStorage.setItem(STORE_DATA_KEY, JSON.stringify(result.data));
                return result.data;
            });
        } catch(error) {
            throw new Error("Could not load from api.", {
                cause: error
            });
        }
    }, []));

    const [done, setDone] = useState(() => checkLoadingToken());

    useEffect(() => {
        if (result === undefined) {
            return;
        }

        if (done === true) {
            return;
        }
        
        const interval = setInterval(async () => {
            const response = await client.get(result.doneUrl);
            if (response.status !== 200) {
                return;
            }

            localStorage.removeItem(STORE_DATA_KEY);
            localStorage.setItem(STORE_TOKEN_DATA_KEY, response.data.token);

            client.defaults.headers.common.Authorization = `Bearer ${response.data.token}`;

            setDone(true);
        }, 1000);

        return () => {
            clearInterval(interval);
        }
    }, [result, done]);


    if (!done) {
        return <div>
            <a href={result?.loginUrl} target="_blank" rel="noopener noreferrer">Login: with oidc</a>
        </div>
    }

    return children;
}

export const LoginBoundary: FunctionComponent<PropsWithChildren> = ({
    children
}) => {
    return (
        <RetryErrorBoundary automatic={false}>
            <RequestSession>
                {children}
            </RequestSession>
        </RetryErrorBoundary>
    )
}