package patches.buildTypes

import jetbrains.buildServer.configs.kotlin.*
import jetbrains.buildServer.configs.kotlin.buildFeatures.CommitStatusPublisher
import jetbrains.buildServer.configs.kotlin.buildFeatures.PullRequests
import jetbrains.buildServer.configs.kotlin.buildFeatures.commitStatusPublisher
import jetbrains.buildServer.configs.kotlin.buildFeatures.pullRequests
import jetbrains.buildServer.configs.kotlin.buildSteps.DotnetBuildStep
import jetbrains.buildServer.configs.kotlin.buildSteps.dotnetBuild
import jetbrains.buildServer.configs.kotlin.buildSteps.script
import jetbrains.buildServer.configs.kotlin.ui.*

/*
This patch script was generated by TeamCity on settings change in UI.
To apply the patch, change the buildType with id = 'Build'
accordingly, and delete the patch script.
*/
changeBuildType(RelativeId("Build")) {
    vcs {
        remove(DslContext.settingsRoot.id!!)
        add(RelativeId("HttpsGithubComTidyBeeTidybeeBackendRefsHeadsMain1"), "+:hub => .")
    }

    expectSteps {
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
    steps {
        update<DotnetBuildStep>(0) {
            name = "build tidybee-hub"
            clearConditions()
            projects = "hub/tidybee-hub.csproj"
            configuration = "Release"
            dockerImagePlatform = DotnetBuildStep.ImagePlatform.Linux
            param("verbosity", "")
        }
        items.removeAt(1)
    }

    features {
        val feature1 = find<PullRequests> {
            pullRequests {
                provider = github {
                    authType = vcsRoot()
                    filterAuthorRole = PullRequests.GitHubRoleFilter.MEMBER
                    ignoreDrafts = true
                }
            }
        }
        feature1.apply {
            vcsRootExtId = "TidybeeHub_HttpsGithubComTidyBeeTidybeeBackendRefsHeadsMain1"
            provider = github {
                serverUrl = ""
                authType = token {
                    token = "credentialsJSON:0f816045-3db7-4a38-893d-b59e0b71a889"
                }
                filterSourceBranch = ""
                filterTargetBranch = "+:refs/heads/main"
                filterAuthorRole = PullRequests.GitHubRoleFilter.MEMBER
                ignoreDrafts = true
            }
        }
        val feature2 = find<CommitStatusPublisher> {
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
        feature2.apply {
            param("github_oauth_user", "")
        }
    }
}
