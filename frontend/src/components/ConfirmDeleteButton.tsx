import { useCallback, useMemo, useState, type FunctionComponent, type MouseEvent, type PointerEvent } from "react";
import { client } from "../api";

type ConfirmDeleteButtonProps = {
    packageName: string;
}

export const ConfirmDeleteButton: FunctionComponent<ConfirmDeleteButtonProps> = ({
    packageName
}) => {

    const [removed, setRemoved] = useState(false);
    const [showDetails, setShowDetails] = useState(false);
    const doDelete = useCallback(async () => {
        await client.delete(`/-/api/delete/${encodeURIComponent(packageName)}`);
        setRemoved(true);
    }, []);

    const onClick = useCallback((event: MouseEvent) => {
        if (event.shiftKey) {
            doDelete();
            return;
        }

        setShowDetails(true);
    }, [ packageName ]);

    const realPackageName = useMemo(() => decodeURIComponent(packageName), [ packageName ]);

    if (removed) {
        return null;
    }

    return <div className="bg-gray-500 rounded-xl">
        <button type="button" onClick={onClick}>{realPackageName}</button>
        {showDetails && (
            <button type="button" onClick={doDelete}>Delete now {realPackageName}!</button>
        )}
    </div>
}