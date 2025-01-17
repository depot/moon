use moon_config::WorkspaceConfig;
use moon_lang_node::node;
use moon_toolchain::{Downloadable, Executable, Installable, Toolchain};
use predicates::prelude::*;
use std::env;
use std::path::PathBuf;

async fn create_node_tool() -> (Toolchain, assert_fs::TempDir) {
    let base_dir = assert_fs::TempDir::new().unwrap();

    let mut config = WorkspaceConfig::default();

    config.node.version = String::from("1.0.0");

    let toolchain = Toolchain::create_from_dir(base_dir.path(), &env::temp_dir(), &config)
        .await
        .unwrap();

    (toolchain, base_dir)
}

fn get_download_file() -> String {
    node::get_download_file("1.0.0").unwrap()
}

fn create_shasums(hash: &str) -> String {
    format!("{hash}  node-v1.0.0-darwin-arm64.tar.gz\n{hash}  node-v1.0.0-darwin-x64.tar.gz\n{hash}  node-v1.0.0-linux-x64.tar.gz\n{hash}  node-v1.0.0-win-x64.zip\n", hash = hash)
}

#[tokio::test]
async fn generates_paths() {
    let (toolchain, temp_dir) = create_node_tool().await;
    let node = toolchain.get_node();

    assert!(predicates::str::ends_with(
        PathBuf::from(".moon")
            .join("tools")
            .join("node")
            .join("1.0.0")
            .to_str()
            .unwrap()
    )
    .eval(node.get_install_dir().unwrap().to_str().unwrap()));

    let bin_path = PathBuf::from(".moon")
        .join("tools")
        .join("node")
        .join("1.0.0")
        .join(node::get_bin_name_suffix("node", "exe", false));

    assert!(predicates::str::ends_with(bin_path.to_str().unwrap())
        .eval(node.get_bin_path().to_str().unwrap()));

    assert!(predicates::str::ends_with(
        PathBuf::from(".moon")
            .join("temp")
            .join("node")
            .join(get_download_file())
            .to_str()
            .unwrap()
    )
    .eval(node.get_download_path().unwrap().to_str().unwrap()));

    temp_dir.close().unwrap();
}

mod download {
    use super::*;
    use mockito::mock;

    #[tokio::test]
    async fn is_downloaded_checks() {
        let (toolchain, temp_dir) = create_node_tool().await;
        let node = toolchain.get_node();

        assert!(!node.is_downloaded().await.unwrap());

        let dl_path = node.get_download_path().unwrap();

        std::fs::create_dir_all(dl_path.parent().unwrap()).unwrap();
        std::fs::write(dl_path, "").unwrap();

        assert!(node.is_downloaded().await.unwrap());

        std::fs::remove_file(dl_path).unwrap();

        temp_dir.close().unwrap();
    }

    #[tokio::test]
    async fn downloads_to_temp_dir() {
        let (toolchain, temp_dir) = create_node_tool().await;
        let node = toolchain.get_node();

        assert!(!node.get_download_path().unwrap().exists());

        let archive = mock(
            "GET",
            format!("/dist/v1.0.0/{}", get_download_file()).as_str(),
        )
        .with_body("binary")
        .create();

        let shasums = mock("GET", "/dist/v1.0.0/SHASUMS256.txt")
            .with_body(create_shasums(
                "9a3a45d01531a20e89ac6ae10b0b0beb0492acd7216a368aa062d1a5fecaf9cd",
            ))
            .create();

        node.download(&toolchain, Some(&mockito::server_url()))
            .await
            .unwrap();

        archive.assert();
        shasums.assert();

        assert!(node.get_download_path().unwrap().exists());

        temp_dir.close().unwrap();
    }

    #[tokio::test]
    #[should_panic(expected = "InvalidShasum")]
    async fn fails_on_invalid_shasum() {
        let (toolchain, temp_dir) = create_node_tool().await;
        let node = toolchain.get_node();

        let archive = mock(
            "GET",
            format!("/dist/v1.0.0/{}", get_download_file()).as_str(),
        )
        .with_body("binary")
        .create();

        let shasums = mock("GET", "/dist/v1.0.0/SHASUMS256.txt")
            .with_body(create_shasums("fakehash"))
            .create();

        node.download(&toolchain, Some(&mockito::server_url()))
            .await
            .unwrap();

        archive.assert();
        shasums.assert();

        assert!(node.get_download_path().unwrap().exists());

        temp_dir.close().unwrap();
    }
}
