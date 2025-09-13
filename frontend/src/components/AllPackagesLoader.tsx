import { useCallback, useMemo, useState, type FunctionComponent } from "react";
import { client } from "../api";
import { ConfirmDeleteButton } from "./Tree/ConfirmDeleteButton";
import { useLoaded } from "../hook/useLoaded";
import { TreeNode } from "./Tree/TreeNode";
import fuzzysort from "fuzzysort";

// const fuzzyMatch = (pattern: string): RegExp => {
//     return new RegExp(pattern.split("").map(s => s.replace(/[.*+?^${}()|[\]\\]/g, "\\$&")).join(".*"), "i");
// }

export const AllPackagesLoader: FunctionComponent = () => {
    
    const [search, setSearch] = useState<string>("");

    const rawPackages = useLoaded(useCallback(async (): Promise<string[]> => {
        try {
            return await client.get("/-/api/all").then((result) => result.data);
        } catch (error) {
            throw new Error("Cannot load api.", {
                cause: error
            });
        }
    }, []));

    const searchTargets = useMemo(() => {
        return Object.fromEntries(rawPackages?.map((prev) => [prev, fuzzysort.prepare(decodeURIComponent(prev))] as const) ?? []);
    }, [ rawPackages ]);

    const searchTargetsInverse = useMemo(() => {
        return Object.fromEntries(rawPackages?.map((prev) => [decodeURIComponent(prev), prev] as const) ?? []);
    }, [ searchTargets ]);

    const packages = useMemo(() => {
        if (search.trim() === '') {
            return rawPackages;
        }

        return fuzzysort.go(search, Object.values(searchTargets), {
            threshold: 0
        }).map((r) => searchTargetsInverse[r.target]);
    }, [search, rawPackages, searchTargets, searchTargetsInverse]);

    const packageGroups = useMemo(() => {
        if (!packages) {
            return {};
        }

        const counters: Record<string, number> = {};
        const results: Record<string, string[]> = {};

        for (const pkgName of packages) {

            results[pkgName] ??= [pkgName];

            for (const subPkgName of packages) {

                if (pkgName == subPkgName) {
                    continue;
                }

                if (!subPkgName.startsWith(pkgName)) {
                    continue;
                }

                const sub = subPkgName.substring(pkgName.length);
                if (!sub.startsWith('/')) {
                    continue;
                }

                results[pkgName].push(subPkgName);
                counters[subPkgName]++;
            }
        }

        for (const key in counters) {
            delete results[key];
        }

        return results;
    }, [ packages ]);

    const all = useMemo(() => Object.entries(packageGroups), [ packageGroups ])

    return <div className="flex flex-col gap-y-6">
        <input value={search} className="bg-gray-400 rounded-3xl mx-4" onChange={e => setSearch(e.target.value)} type="text" placeholder="filter packages..." />
        {all.map(([pgkName, metadatas]) => (
            <TreeNode packageName={pgkName} packageNames={metadatas} all={rawPackages ?? []}>
                {metadatas.sort().map((packageName) => <ConfirmDeleteButton key={packageName} packageName={packageName}/>)}
            </TreeNode>
        ))}
    </div>
}