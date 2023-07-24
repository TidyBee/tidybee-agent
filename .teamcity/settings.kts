import jetbrains.buildServer.configs.kotlin.*
import jetbrains.buildServer.configs.kotlin.buildFeatures.PullRequests
import jetbrains.buildServer.configs.kotlin.buildFeatures.commitStatusPublisher
import jetbrains.buildServer.configs.kotlin.buildFeatures.perfmon
import jetbrains.buildServer.configs.kotlin.buildFeatures.pullRequests
import jetbrains.buildServer.configs.kotlin.buildSteps.DotnetBuildStep
import jetbrains.buildServer.configs.kotlin.buildSteps.dotnetBuild
import jetbrains.buildServer.configs.kotlin.buildSteps.nunit
import jetbrains.buildServer.configs.kotlin.buildSteps.powerShell
import jetbrains.buildServer.configs.kotlin.buildSteps.script
import jetbrains.buildServer.configs.kotlin.triggers.vcs

/*
The settings script is an entry point for defining a TeamCity
project hierarchy. The script should contain a single call to the
project() function with a Project instance or an init function as
an argument.

VcsRoots, BuildTypes, Templates, and subprojects can be
registered inside the project using the vcsRoot(), buildType(),
template(), and subProject() methods respectively.

To debug settings scripts in command-line, run the

    mvnDebug org.jetbrains.teamcity:teamcity-configs-maven-plugin:generate

command and attach your debugger to the port 8000.

To debug in IntelliJ Idea, open the 'Maven Projects' tool window (View
-> Tool Windows -> Maven Projects), find the generate task node
(Plugins -> teamcity-configs -> teamcity-configs:generate), the
'Debug' option is available in the context menu for the task.
*/

version = "2023.05"

project {

    buildType(Build)
    buildType(Build_2)

    params {
        param("env.DOTNET_ROOT", "/usr/lib/dotnet")
    }
}

object Build : BuildType({
    name = "Build"

    params {
        param("env.DOTNET_HOME", "/usr/bin")
    }

    vcs {
        root(DslContext.settingsRoot)
    }

    steps {
        dotnetBuild {
            name = "Build"
            projects = """
                functionnalTests/*.csproj
                unitaryTests/*.csproj
            """.trimIndent()
            logging = DotnetBuildStep.Verbosity.Normal
            dockerImage = "mcr.microsoft.com/dotnet/sdk:7.0"
            param("dotNetCoverage.dotCover.home.path", "%teamcity.tool.JetBrains.dotCover.CommandLineTools.DEFAULT%")
        }
        nunit {
            name = "Test project"
            enabled = false
            nunitPath = "%teamcity.tool.NUnit.Console.3.16.2%"
            includeTests = "unitaryTests/serviceInterfaceTests.cs"
            coverage = dotcover {
            }
        }
        powerShell {
            name = "vstest way"
            enabled = false
            scriptMode = script {
                content = """
                    cd functionnalTests
                    dotnet publish -o out
                    dotnet vstest out/TidyUpSoftware.xUnitTests.dll
                    cd ..
                    dotnet publish -o out
                    dotnet vstest out/TidyUpSoftware.nUnitTests.dll
                """.trimIndent()
            }
        }
        script {
            name = "Tests"
            scriptContent = """
                cd functionnalTests
                dotnet publish -o out
                dotnet vstest out/TidyUpSoftware.xUnitTests.dll
                cd ..
                dotnet publish -o out
                dotnet vstest out/TidyUpSoftware.nUnitTests.dll
            """.trimIndent()
        }
    }

    triggers {
        vcs {
        }
    }

    features {
        perfmon {
        }
        pullRequests {
            provider = github {
                authType = vcsRoot()
                filterAuthorRole = PullRequests.GitHubRoleFilter.MEMBER
                ignoreDrafts = true
            }
        }
        commitStatusPublisher {
            publisher = github {
                githubUrl = "https://api.github.com"
                authType = personalToken {
                    token = "credentialsJSON:0f816045-3db7-4a38-893d-b59e0b71a889"
                }
            }
            param("github_oauth_user", "Cavonstavant")
        }
    }
})

object Build_2 : BuildType({
    name = "Build (backup)"

    params {
        param("env.DOTNET_HOME", "/usr/bin")
    }

    vcs {
        root(DslContext.settingsRoot)
    }

    steps {
        dotnetBuild {
            name = "Build"
            enabled = false
            projects = """
                functionnalTests/*.csproj
                unitaryTests/*.csproj
            """.trimIndent()
            logging = DotnetBuildStep.Verbosity.Normal
            dockerImage = "mcr.microsoft.com/dotnet/sdk:7.0"
            param("dotNetCoverage.dotCover.home.path", "%teamcity.tool.JetBrains.dotCover.CommandLineTools.DEFAULT%")
        }
        nunit {
            name = "Test project"
            enabled = false
            nunitPath = "%teamcity.tool.NUnit.Console.3.16.2%"
            includeTests = "unitaryTests/serviceInterfaceTests.cs"
            coverage = dotcover {
            }
        }
        powerShell {
            name = "vstest way"
            enabled = false
            scriptMode = script {
                content = """
                    cd functionnalTests
                    dotnet publish -o out
                    dotnet vstest out/TidyUpSoftware.xUnitTests.dll
                    cd ..
                    dotnet publish -o out
                    dotnet vstest out/TidyUpSoftware.nUnitTests.dll
                """.trimIndent()
            }
        }
        script {
            name = "cli way"
            enabled = false
            scriptContent = """
                cd functionnalTests
                dotnet publish -o out
                dotnet vstest out/TidyUpSoftware.xUnitTests.dll
                cd ..
                dotnet publish -o out
                dotnet vstest out/TidyUpSoftware.nUnitTests.dll
            """.trimIndent()
        }
    }

    triggers {
        vcs {
            branchFilter = "16-backtestingserviceinterface"
        }
    }

    features {
        perfmon {
        }
        pullRequests {
            provider = github {
                authType = vcsRoot()
                filterAuthorRole = PullRequests.GitHubRoleFilter.MEMBER
                ignoreDrafts = true
            }
        }
        commitStatusPublisher {
            publisher = github {
                githubUrl = "https://api.github.com"
                authType = personalToken {
                    token = "credentialsJSON:0f816045-3db7-4a38-893d-b59e0b71a889"
                }
            }
            param("github_oauth_user", "Cavonstavant")
        }
    }
})
