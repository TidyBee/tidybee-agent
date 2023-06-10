# TidyUpSoftware Unit Tests

Unitary tests in C# .NET.

## Publish
If output directory isn't specified, it defaults to ./bin/Debug/net\<local SDK version>/
```
dotnet publish -o out
```

## Run
```
vstest.console.exe out/TidyUpSoftware.nUnitTests.dll
```

Alternatively, you can run the tests from Visual Studio. Open the csproj file in Visual Studio, then run the tests from the Test Explorer which can be opened with the hotkey (Ctrl + E, T) or from the test dropdown menu.

You can also use the dotnet CLI :
```
dotnet vstest out/TidyUpSoftware.nUnitTests.dll
```