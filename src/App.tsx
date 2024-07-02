import { invoke } from "@tauri-apps/api/tauri";
import { open } from "@tauri-apps/api/dialog";
import { emit, listen } from "@tauri-apps/api/event";

import "./App.css";
import React, { useEffect, useState } from "react";
import ConfettiExplosion from "react-confetti-explosion";

export enum DownloadStatus {
  Downloading = "Downloading",
  Downloaded = "Downloaded",
  Failed = "Failed",
}

export type Download = {
  url: string;
  status: DownloadStatus;
};

export type DownloadPayload = Download[];
export type BulkDownloadPayload = {
  urls: DownloadPayload;
  // janky way to make it remove the element
  // TODO: make it not janky :blobcatcozy:
  shouldRemove: boolean;
};

function Download(props: { download: Download }) {
  const { download } = props;
  const statusString =
    download.status == DownloadStatus.Downloading
      ? "Downloading"
      : download.status == DownloadStatus.Downloaded
      ? "Downloaded"
      : download.status == DownloadStatus.Failed
      ? "Failed"
      : "huh";
  const downloadStatusStyle =
    download.status == DownloadStatus.Downloading
      ? "download-status-pending"
      : download.status == DownloadStatus.Downloaded
      ? "download-status-success"
      : download.status == DownloadStatus.Failed
      ? "download-status-failed"
      : "huh";

  return (
    <div className="download">
      <p className="download-url">{download.url}</p>
      <b className={"download-status " + downloadStatusStyle}>{statusString}</b>
      <button
        className="download-close"
        onClick={() => {
          console.log("meow");
          emit("url-updated", {
            ...download,
            shouldRemove: true,
          });
        }}
      >
        Remove
      </button>
    </div>
  );
}

function DownloadsList(props: { downloads: Download[] }) {
  const { downloads } = props;
  const displayedDownloads = downloads.map((download) => (
    <Download download={download} key={download.url} />
  ));

  if (downloads.length != 0) {
    return <div className="download-list">{displayedDownloads}</div>;
  } else {
    return (
      <div className="download-list">
        <p>No currently active downloads :c</p>
      </div>
    );
  }
}

function ActiveDownloadsHeader(props: {
  filesToDownload: number;
  filesDownloaded: number;
}) {
  const { filesToDownload, filesDownloaded } = props;

  if (filesToDownload == filesDownloaded && filesToDownload != 0) {
    return (
      <h3>
        Active downloads (All {filesToDownload} files have been downloaded! 🎉)
      </h3>
    );
  }

  if (filesToDownload && filesDownloaded != 0) {
    return (
      <h3>
        Active downloads ({filesToDownload}/{filesDownloaded} downloaded)
      </h3>
    );
  } else {
    return <h3>Active downloads</h3>;
  }
}

function App() {
  const [downloads, setDownloads] = useState<Download[]>([]);
  const [filesToDownload, setFilesToDownload] = useState(0);
  const [filesDownloaded, setFilesDownloaded] = useState(0);

  const [isExploding, setIsExploding] = useState(false);

  useEffect(() => {
    const bulkUrls = listen("bulk-urls", (event) => {
      const payload = event.payload as BulkDownloadPayload;
      const { urls } = payload;
      setFilesToDownload(urls.length);
      setDownloads([...downloads, ...urls]);
    });

    const urlUpdated = listen("url-updated", (event) => {
      const download = event.payload as Download;

      if (download.status == DownloadStatus.Downloaded) {
        setFilesDownloaded(filesDownloaded + 1);
      }

      const newDownloads = downloads
        .map((_download) => {
          if (_download.url == download.url) {
            _download.status = download.status;
          }
          return _download;
          // remove the dirty ones
        })
        .filter((_download) => {
          const shouldRemove =
            "shouldRemove" in download && _download.url == download.url;

          if (!shouldRemove) {
            return _download;
          }
        });

      setDownloads(newDownloads);
    });

    return () => {
      bulkUrls.then((unlisten) => unlisten());
      urlUpdated.then((unlisten) => unlisten());
    };
  });

  useEffect(() => {
    if (filesDownloaded == filesToDownload && filesDownloaded != 0) {
      setIsExploding(true);
      setTimeout(() => {
        setIsExploding(false);
      }, 6000);
    }
  }, [filesToDownload, filesDownloaded]);

  async function openFile() {
    const selected = await open({
      filters: [
        {
          name: "JSON files",
          extensions: ["json"],
        },
      ],
    });

    if (selected == null) {
      alert("pick a file silly");
    } else {
      await invoke("download_urls", {
        filePath: selected,
      });
    }
  }

  return (
    <div className="container">
      <>{isExploding && <ConfettiExplosion />}</>
      <h1>the world's best url downloader</h1>
      <h2>(real)</h2>

      <ActiveDownloadsHeader
        filesToDownload={filesToDownload}
        filesDownloaded={filesDownloaded}
      />
      <DownloadsList downloads={downloads} />
      <br />

      <button onClick={openFile}>Pick a JSON file with urls</button>
    </div>
  );
}

export default App;
