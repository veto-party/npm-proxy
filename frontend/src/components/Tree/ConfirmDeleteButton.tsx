import { useCallback, useMemo, useState, type FunctionComponent, type MouseEvent, type PointerEvent } from "react";
import { client } from "../../api";
import { useFastDelete } from "../../hook/useFastDelete";

type ConfirmDeleteButtonProps = {
    packageName: string;
}

export const ConfirmDeleteButton: FunctionComponent<ConfirmDeleteButtonProps> = ({
    packageName
}) => {

   const [details, onClick, removed] = useFastDelete(useCallback(async () => {
        await client.delete(`/-/api/delete/${packageName}`);
    }, [ packageName ]));

    const realPackageName = useMemo(() => decodeURIComponent(packageName), [ packageName ]);

    if (removed) {
        return null;
    }

    return <div className="bg-gray-500 rounded-xl px-2">
        <button type="button" onClick={onClick}>{realPackageName}</button>
        {details}
    </div>
}