export class DataWriteOptions {

}

/** Basic obsidian abstraction for any file or folder in a vault. */
export abstract class TAbstractFile {
    /**
     * @public
     */
    vault: Vault;
    /**
     * @public
     */
    path: string;
    /**
     * @public
     */
    name: string;
    /**
     * @public
     */
    parent: TFolder;
}

/** Tracks file created/modified time as well as file system size. */
export interface FileStats {
    /** @public */
    ctime: number;
    /** @public */
    mtime: number;
    /** @public */
    size: number;
}

/** A regular file in the vault. */
export class TFile extends TAbstractFile {
    stat: FileStats;
    basename: string;
    extension: string;
}

/** Tracks file created/modified time as well as file system size. */
export interface FileStats {
    /** @public */
    ctime: number;
    /** @public */
    mtime: number;
    /** @public */
    size: number;
}

/** A folder in the vault. */
export class TFolder extends TAbstractFile {
    children: TAbstractFile[];

    isRoot(): boolean {
        return false;
    }
}

export class Vault {
    getConfig(key: string) { }

    create(path: string, data: string, options?: DataWriteOptions | undefined): Promise<TFile> {
        return Promise.resolve(new TFile());
    }
}

export default {
    Vault
}