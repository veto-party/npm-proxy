import { useCallback, type FunctionComponent } from "react";
import { client } from "../api";
import { ConfirmDeleteButton } from "./ConfirmDeleteButton";
import { useLoaded } from "../hook/useLoaded";

export const AllPackagesLoader: FunctionComponent = () => {
    
    const packages = useLoaded(useCallback(async (): Promise<string[]> => {
        try {
            return await client.get("/-/api/all").then((result) => result.data);
        } catch (error) {
            throw new Error("Cannot load api.", {
                cause: error
            });
        }
    }, []));

    return <div className="flex flex-col gap-y-6">
        {packages?.map((packageName) => (
            <ConfirmDeleteButton key={packageName} packageName={packageName}/>
        ))}
    </div>
}