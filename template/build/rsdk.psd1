@{
    # Module manifest version
    ModuleVersion = '1.0.0'

    # Author
    Author = 'Francis Lalonde'

    # Description of the module
    Description = 'PowerShell module that wraps rsdk.exe commands.'

    # Functions to export
    FunctionsToExport = @('Invoke-RsdkCommand')

    # Script module to load (psm1 file)
    RootModule = 'rsdk.psm1'

    # PowerShell version compatibility
    PowerShellVersion = '5.1'

    # Optional: CompanyName, Copyright, etc.
}
