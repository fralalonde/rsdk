# rsdk - Native JVM tools manager

`rsdk` is a native command-line JVM tool manager, similar to SDKMAN.

## Installation via Scoop

1. Add the custom Scoop bucket (self-hosted from this repo):

    ```bash
    scoop bucket add rsdk-bucket https://github.com/yourusername/rsdk
    ```

2. Install `rsdk`:

    ```bash
    scoop install rsdk
    ```

## Manual Installation

You can manually install the latest version of `rsdk` by following these steps:

1. Download the latest `rsdk.exe` from the [releases page](https://github.com/yourusername/rsdk/releases).
2. Move it to a folder like `C:\Tools` or any directory you prefer.
3. Add that folder to your `PATH`:
    - Right-click **This PC** and select **Properties**.
    - Select **Advanced system settings**.
    - Click **Environment Variables**.
    - Find the **Path** variable under **System variables**, select it, and click **Edit**.
    - Click **New** and add the path to the folder where `rsdk.exe` is stored.
4. Open a new Command Prompt or PowerShell window and verify installation: `rsdk --version`

## Usage

### Install a Candidate

``rsdk install <candidate> [version] [--install-path <path>]``

Example: ``rsdk install java 17``

### Uninstall a Candidate

``rsdk uninstall <candidate> <version>``

Example: ``rsdk uninstall java 17``

### List Available Versions of a Candidate

``rsdk list <candidate>``

Example: ``rsdk list java``

### Use a Specific Version of a Candidate

``rsdk use <candidate> <version>``

Example: ``rsdk use java 17``

### Set a Version of a Candidate as Default

``rsdk default <candidate> <version>``

Example: ``rsdk default java 17``

### Enable or Disable Offline Mode

``rsdk offline enable``

or

``rsdk offline disable``

### Show Help

To display a full list of commands and options:

``rsdk --help``
