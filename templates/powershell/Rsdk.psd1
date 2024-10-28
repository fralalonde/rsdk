@{
    GUID = "51655fae-3ed7-48fe-98e2-4a01e9e6957a"
    Author = "Rsdk"
    CompanyName = "Rsdk"
    Description = 'Rsdk JVM tool manager'
    ModuleVersion = "1.0.0"
    PowerShellVersion = "5.1"
    FunctionsToExport = @('Invoke-Rsdk', 'Install-Rsdk', 'Uninstall-Rsdk', 'List-Rsdk', 'Select-Rsdk', 'Reset-Rsdk')
    CmdletsToExport = @()
    AliasesToExport = @()
    RootModule = 'Rsdk.psm1'
    HelpInfoURI = 'https://github.com/fralalonde/rsdk'
}