use std::{fs, mem::forget};
use zed_extension_api::{
    current_platform, download_file, latest_github_release, make_file_executable,
    register_extension, set_language_server_installation_status, Architecture, Command,
    DownloadedFileType, Extension, GithubReleaseOptions, LanguageServerId,
    LanguageServerInstallationStatus, Os, Result, Result as ZedResult, Worktree,
};

struct C3Extension {
    bin_path: Option<String>,
}

impl C3Extension {
    fn language_server_binary(
        &mut self,
        language_server_id: &LanguageServerId,
    ) -> ZedResult<String> {
        if let Some(path) = &self.bin_path {
            if fs::metadata(path).map_or(false, |stat| stat.is_file()) {
                return Ok(path.clone());
            }
        }

        set_language_server_installation_status(
            &language_server_id,
            &LanguageServerInstallationStatus::CheckingForUpdate,
        );

        let release = latest_github_release(
            "pherrymason/c3-lsp",
            GithubReleaseOptions {
                require_assets: true,
                pre_release: false,
            },
        )?;
        let (platform, _arch) = current_platform();
        // let asset_name = format!(
        //     "{os}-{arc}-c3lsp.zip",
        //     os = match platform {
        //         Os::Mac => "darwin",
        //         Os::Linux => "linux",
        //         Os::Windows => "windows",
        //     },
        //     arc = match arch {
        //         Architecture::X8664 => "amd64",
        //         Architecture::Aarch64 => "arm64",
        //         a => todo!("{a:?}"),
        //     },
        // );
        let asset_name = match platform {
            // temp as maintainer of lsp is very smart
            Os::Windows => "windows-amd64",
            Os::Mac => "darwin-amd64-c3lsp",
            Os::Linux => "linux-amd64-c3lsp",
        };
        let asset_file = format!("{asset_name}.zip");
        let asset = release
            .assets
            .iter()
            .find(|asset| asset.name == asset_file)
            .ok_or_else(|| format!("no asset found matching {asset_file:?}"))?;
        let version_dir = format!("c3-language-server-{}", release.version);
        let binary_path = format!("{version_dir}/{asset_name}/c3_language_server_wrapper");

        if !fs::metadata(&binary_path).map_or(false, |stat| stat.is_file()) {
            set_language_server_installation_status(
                &language_server_id,
                &LanguageServerInstallationStatus::Downloading,
            );

            download_file(&asset.download_url, &version_dir, DownloadedFileType::Zip)
                .map_err(|e| format!("failed to download file: {e}"))?;
            make_file_executable(&binary_path)?;

            let entries =
                fs::read_dir(".").map_err(|e| format!("failed to list working directory {e}"))?;

            for entry in entries {
                let entry = entry.map_err(|e| format!("failed to load directory entry {e}"))?;

                if entry.file_name().to_str() != Some(&version_dir) {
                    fs::remove_dir_all(&entry.path()).ok();
                }
            }
        }

        self.bin_path = Some(binary_path.clone());

        Ok(binary_path)
    }
}

impl Extension for C3Extension {
    fn new() -> Self
    where
        Self: Sized,
    {
        Self { bin_path: None }
    }

    fn language_server_command(
        &mut self,
        language_server_id: &LanguageServerId,
        _worktree: &Worktree,
    ) -> Result<Command> {
        Ok(Command {
            command: self.language_server_binary(language_server_id)?,
            args: Vec::new(),
            env: Vec::new(),
        })
    }
}

register_extension!(C3Extension);
