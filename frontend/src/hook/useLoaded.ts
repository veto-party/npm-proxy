import { useEffect, useState } from "react"

type LoadedResult<T> = {
    type: 'success';
    value: T;
} | {
    type: 'error';
    error: any;
}

export const useLoaded = <T>(cb: () => Promise<T>) => {
    const [result, setResult] = useState<LoadedResult<T>>();
    
    useEffect(() => {
        try {
            cb().then((value) => {
                setResult({
                    type: 'success',
                    value
                });
            }, (error) => {
                setResult({
                   type: 'error',
                   error 
                });
            })
        } catch (error) {
            setResult({
                type: 'error',
                error
            })
        }
    }, [cb, setResult]);

    useEffect(() => {
        if (result === undefined) {
            return;
        }

        if (result.type === 'success') {
            return;
        }

        throw result.error;
    }, [result]);


    if (result?.type === "success") {
        return result.value;
    }
    
    return undefined;
}