import { useCallback, useMemo, useState, type MouseEvent } from "react"

export const useFastDelete = (callback: () => Promise<any>) => {
    const [showDelete, setShowDelete] = useState(false);
    const [deleted, setDeleted] = useState(false);

    const doDelete = useCallback(() => {
        callback().then(() => setDeleted(true));
    }, [callback, setDeleted]);

    const deleteDetails = useMemo(() => {

        if (!showDelete) {
            return <></>;
        }

        return (
            <div>
                <button onClick={doDelete}>Do delete now</button>
            </div>
        );
    }, [ showDelete, doDelete ]);

    const givenCallback = useCallback((event: MouseEvent) => {
        if (event.shiftKey) {
            doDelete();
        }

        setShowDelete(true);
    }, [ doDelete, setShowDelete ]);

    return [deleteDetails, givenCallback, deleted] as const;
}