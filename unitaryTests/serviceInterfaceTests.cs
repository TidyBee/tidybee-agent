using NUnit.Framework;

namespace TidyUpSoftware.nUnitTests
{
    public class serviceInterfaceTests
    {
        [SetUp]
        public void Setup()
        {
            // serviceInterface _serviceInterface = new ServiceInterface();
            // string testFolder1;
            // string testFolder2;
            // string notAllowedFolder;
        }

        [Test]
        public void getFileListTestSucceed()
        {
            // Assign
            List<string> result = new List<string>();
            result.Add("testfile1.txt");
            result.Add("testfile2.txt");
            result.Add("testfile3.txt");
            // _serviceInterface.setFolderPath(testFolder1);

            // Act
            List<string> fileList = new List<string>();
            // fileList = _serviceInterface.getFileList();

            // Assert
            Assert.AreEqual(fileList, result);
        }

        [Test]
        public void getFileListTestSucceedEmpty()
        {
            // Assign
            List<string> result = new List<string>();
            // _serviceInterface.setFolderPath(testFolder2);

            // Act
            List<string> fileList = new List<string>();
            // fileList = _serviceInterface.getFileList();

            // Assert
            Assert.AreEqual(fileList, result);
        }

        [Test]
        public void getFileListTestError()
        {
            // Assign
            bool catchException = false;
            // _serviceInterface.setFolderPath("asdasdasd");

            // Act
            try
            {
                // _serviceInterface.getFileList();
            }
            catch (Exception ex)
            {
                catchException = true;
            }

            // Assert
            Assert.AreEqual(catchException, true);
        }

        [Test]
        public void getFileListTestNotAllowed()
        {
            // Assign
            bool catchException = false;
            // _serviceInterface.setFolderPath(notAllowedFolder);

            // Act
            try
            {
                // _serviceInterface.getFileList();
            }
            catch (Exception ex)
            {
                catchException = true;
            }

            // Assert
            Assert.AreEqual(catchException, true);
        }

        [Test]
        public void getFolderListTestSucceed()
        {
            // Assign
            List<string> result = new List<string>();
            result.Add("testFolder");
            result.Add("testFolder2");
            result.Add("testFolder3");
            // _serviceInterface.setFolderPath(testFolder1);

            // Act
            List<string> folderList = new List<string>();
            // folderList = _serviceInterface.getFolderList();

            // Assert
            Assert.AreEqual(folderList, result);
        }

        [Test]
        public void getFolderListTestSucceedEmpty()
        {
            // Assign
            List<string> result = new List<string>();
            // _serviceInterface.setFolderPath(testFolder2);

            // Act
            List<string> folderList = new List<string>();
            // folderList = _serviceInterface.getFolderList();

            // Assert
            Assert.AreEqual(folderList, result);
        }

        [Test]
        public void getFolderListTestError()
        {
            // Assign
            bool catchException = false;
            // _serviceInterface.setFolderPath("asdasd");

            // Act
            try
            {
                // _serviceInterface.getFolderList();
            }
            catch (Exception ex)
            {
                catchException = true;
            }

            // Assert
            Assert.AreEqual(catchException, true);
        }

        [Test]
        public void getFolderListTestNotAllowed()
        {
            // Assign
            bool catchException = false;
            // _serviceInterface.setFolderPath(notAllowedFolder);

            // Act
            try
            {
                // _serviceInterface.getFolderList();
            }
            catch (Exception ex)
            {
                catchException = true;
            }

            // Assert
            Assert.AreEqual(catchException, true);
        }

        [Test]
        public void getNumberOfFilesTestSucceed()
        {
            // Assign
            int result = 3;
            // _serviceInterface.setFolderPath(testFolder1);

            // Act
            int fileCount = 0;
            // fileCount = _serviceInterface.getNumberOfFiles();

            // Assert
            Assert.AreEqual(fileCount, result);
        }

        [Test]
        public void getNumberOfFilesTestSucceedEmpty()
        {
            // Assign
            int result = 0;
            // _serviceInterface.setFolderPath(testFolder2);

            // Act
            int fileCount = 12;
            // fileCount = _serviceInterface.getNumberOfFiles();

            // Assert
            Assert.AreEqual(fileCount, result);
        }

        [Test]
        public void getNumberOfFilesTestError()
        {
            // Assign
            bool catchException = false;
            // _serviceInterface.setFolderPath("asdasd");

            // Act
            try
            {
                // _serviceInterface.getNumberOfFiles();
            }
            catch (Exception ex)
            {
                catchException = true;
            }

            // Assert
            Assert.AreEqual(catchException, true);
        }

        [Test]
        public void getNumberOfFoldersTestSucceed()
        {
            // Assign
            int result = 3;
            // _serviceInterface.setFolderPath(testFolder1);

            // Act
            int folderCount = 0;
            // folderCount = _serviceInterface.getNumberOfFolders();

            // Assert
            Assert.AreEqual(folderCount, result);
        }

        [Test]
        public void getNumberOfFoldersTestSucceedEmpty()
        {
            // Assign
            int result = 0;
            // _serviceInterface.setFolderPath(testFolder2);

            // Act
            int folderCount = 12;
            // folderCount = _serviceInterface.getNumberOfFolders();

            // Assert
            Assert.AreEqual(folderCount, result);
        }

        [Test]
        public void getNumberOfFoldersTestError()
        {
            // Assign
            bool catchException = false;
            // _serviceInterface.setFolderPath("asdasd");

            // Act
            try
            {
                // _serviceInterface.getNumberOfFolders();
            }
            catch (Exception ex)
            {
                catchException = true;
            }

            // Assert
            Assert.AreEqual(catchException, true);
        }

        [Test]
        public void getNumberOfFoldersTestNotAllowed()
        {
            // Assign
            bool catchException = false;
            // _serviceInterface.setFolderPath(notAllowedFolder);

            // Act
            try
            {
                // _serviceInterface.getNumberOfFolders();
            }
            catch (Exception ex)
            {
                catchException = true;
            }

            // Assert
            Assert.AreEqual(catchException, true);
        }

        [Test]
        public void getAccessListMemberTestSucceed()
        {
            // Assign
            List<string> result = new List<string>();
            result.Add("user1");
            result.Add("user2");
            result.Add("user3");
            // _serviceInterface.setFolderPath(testFolder1);

            // Act
            List<string> fileList = new List<string>();
            // fileList = _serviceInterface.getAccessListMember();

            // Assert
            Assert.AreEqual(fileList, result);
        }

        [Test]
        public void getAccessListMemberTestSucceedEmpty()
        {
            // Assign
            List<string> result = new List<string>();
            // _serviceInterface.setFolderPath(testFolder2);

            // Act
            List<string> fileList = new List<string>();
            // fileList = _serviceInterface.getAccessListMember();

            // Assert
            Assert.AreEqual(fileList, result);
        }

        [Test]
        public void getAccessListMemberTestError()
        {
            // Assign
            bool catchException = false;
            // _serviceInterface.setFolderPath("asdasdasd");

            // Act
            try
            {
                // _serviceInterface.getAccessListMember();
            }
            catch (Exception ex)
            {
                catchException = true;
            }

            // Assert
            Assert.AreEqual(catchException, true);
        }

        [Test]
        public void getAccessListMemberTestNotAllowed()
        {
            // Assign
            bool catchException = false;
            // _serviceInterface.setFolderPath(notAllowedFolder);

            // Act
            try
            {
                // _serviceInterface.getAccessListMember();
            }
            catch (Exception ex)
            {
                catchException = true;
            }

            // Assert
            Assert.AreEqual(catchException, true);
        }

        [Test]
        public void getUserTestSucceed()
        {
            // Assign
            string result = "user";
            // _serviceInterface.setUser("user");

            // Act
            string user = "";
            // user = _serviceInterface.getUser();

            // Assert
            Assert.AreEqual(user, result);
        }

        [Test]
        public void getFolderPathTest()
        {
            // Assign
            string result = "testFolder";
            // _serviceInterface.setFolderPath("testFolder");

            // Act
            string folderPath = "";
            // folderPath = _serviceInterface.getFolderPath();

            // Assert
            Assert.AreEqual(folderPath, result);
        }

        [Test]
        public void isTheUserAllowedHereTestSucceed()
        {
            // Assign
            // _serviceInterface.setFolderPath(testFolder1);

            // Act
            bool isAllowed = false;
            // isAllowed = _serviceInterface.isTheUserAllowedHere();

            // Assert
            Assert.AreEqual(isAllowed, true);
        }

        [Test]
        public void isTheUserAllowedHereTestSucceedFalse()
        {
            // Assign
            // _serviceInterface.setFolderPath(notAllowedFolder);

            // Act
            bool isAllowed = true;
            // isAllowed = _serviceInterface.isTheUserAllowedHere();

            // Assert
            Assert.AreEqual(isAllowed, false);
        }

        [Test]
        public void isTheUserAllowedHereTestError()
        {
            // Assign
            bool catchException = false;
            // _serviceInterface.setFolderPath(notAllowedFolder);

            // Act
            try {
                // _serviceInterface.isTheUserAllowedHere();
            }
            catch (Exception ex)
            {
                catchException = true;
            }

            // Assert
            Assert.AreEqual(catchException, true);
        }

        [Test]
        public void addShadowFilesTestSucceed()
        {
            // Assign
            List<string> result = new List<string>();
            result.Add("testfile1.txt");
            result.Add("testfile2.txt");
            result.Add("testfile3.txt");
            result.Add("shadow1.txt");
            result.Add("shadow2.txt");
            List<string> shadows = new List<string>();
            shadows.Add("shadow1.txt");
            shadows.Add("shadow2.txt");
            // _serviceInterface.setFolderPath(testFolder1);

            // Act
            // _serviceInterface.addShadowFiles(shadows);
            List<string> fileList = new List<string>();
            // fileList = _serviceInterface.getFileList();


            // Assert
            Assert.AreEqual(fileList, result);
        }

        [Test]
        public void addShadowFilesTestEmpty()
        {
            // Assign
            List<string> result = new List<string>();
            result.Add("testfile1.txt");
            result.Add("testfile2.txt");
            result.Add("testfile3.txt");
            List<string> shadows = new List<string>();
            // _serviceInterface.setFolderPath(testFolder1);

            // Act
            // _serviceInterface.addShadowFiles(shadows);
            List<string> fileList = new List<string>();
            // fileList = _serviceInterface.getFileList();


            // Assert
            Assert.AreEqual(fileList, result);
        }

        [Test]
        public void addShadowFilesTestError()
        {
            // Assign
            bool catchException = false;
            List<string> shadows = new List<string>();
            shadows.Add("shadow1.txt");
            shadows.Add("shadow2.txt");
            // _serviceInterface.setFolderPath("asdasd");

            // Act
            try {
                // _serviceInterface.addShadowFiles(shadows);
            }
            catch (Exception ex)
            {
                catchException = true;
            }

            // Assert
            Assert.AreEqual(catchException, true);
        }

        [Test]
        public void addShadowFilesTestNotAllowed()
        {
            // Assign
            bool catchException = false;
            List<string> shadows = new List<string>();
            shadows.Add("shadow1.txt");
            shadows.Add("shadow2.txt");
            // _serviceInterface.setFolderPath(notAllowedFolder);

            // Act
            try {
                // _serviceInterface.addShadowFiles(shadows);
            }
            catch (Exception ex)
            {
                catchException = true;
            }

            // Assert
            Assert.AreEqual(catchException, true);
        }

        [Test]
        public void deleteShadowFilesTestSucceed()
        {
            // Assign
            List<string> result = new List<string>();
            result.Add("testfile1.txt");
            result.Add("testfile2.txt");
            result.Add("testfile3.txt");
            List<string> shadows = new List<string>();
            shadows.Add("shadow1.txt");
            shadows.Add("shadow2.txt");
            // _serviceInterface.setFolderPath(testFolder1);

            // Act
            // _serviceInterface.addShadowFiles(shadows);
            // _serviceInterface.deleteShadowFiles(shadows);
            List<string> fileList = new List<string>();
            // fileList = _serviceInterface.getFileList();


            // Assert
            Assert.AreEqual(fileList, result);
        }

        [Test]
        public void deleteShadowFilesTestEmpty()
        {
            // Assign
            List<string> result = new List<string>();
            result.Add("testfile1.txt");
            result.Add("testfile2.txt");
            result.Add("testfile3.txt");
            result.Add("shadow1.txt");
            result.Add("shadow2.txt");
            List<string> shadows = new List<string>();
            shadows.Add("shadow1.txt");
            shadows.Add("shadow2.txt");
            List<string> empty = new List<string>();
            // _serviceInterface.setFolderPath(testFolder1);

            // Act
            // _serviceInterface.addShadowFiles(shadows);
            // _serviceInterface.deleteShadowFiles(empty);
            List<string> fileList = new List<string>();
            // fileList = _serviceInterface.getFileList();


            // Assert
            Assert.AreEqual(fileList, result);
        }

        [Test]
        public void deleteShadowFilesTestNotPresent()
        {
            // Assign
            List<string> result = new List<string>();
            result.Add("testfile1.txt");
            result.Add("testfile2.txt");
            result.Add("testfile3.txt");
            List<string> shadows = new List<string>();
            shadows.Add("shadow1.txt");
            shadows.Add("shadow2.txt");
            // _serviceInterface.setFolderPath(testFolder1);

            // Act
            // _serviceInterface.deleteShadowFiles(shadows);
            List<string> fileList = new List<string>();
            // fileList = _serviceInterface.getFileList();

            // Assert
            Assert.AreEqual(fileList, result);
        }

        [Test]
        public void deleteShadowFilesTestError()
        {
            // Assign
            bool catchException = false;
            List<string> shadows = new List<string>();
            shadows.Add("shadow1.txt");
            shadows.Add("shadow2.txt");
            // _serviceInterface.setFolderPath("asdasd");

            // Act
            try {
                // _serviceInterface.deleteShadowFiles(shadows);
            }
            catch (Exception ex)
            {
                catchException = true;
            }

            // Assert
            Assert.AreEqual(catchException, true);
        }

        [Test]
        public void deleteShadowFilesTestNotAllowed()
        {
            // Assign
            bool catchException = false;
            List<string> shadows = new List<string>();
            shadows.Add("shadow1.txt");
            shadows.Add("shadow2.txt");
            // _serviceInterface.setFolderPath(notAllowedFolder);

            // Act
            try {
                // _serviceInterface.deleteShadowFiles(shadows);
            }
            catch (Exception ex)
            {
                catchException = true;
            }

            // Assert
            Assert.AreEqual(catchException, true);
        }

        [Test]
        public void deleteFileTestSucceed()
        {
            // Assign
            List<string> result = new List<string>();
            result.Add("testfile1.txt");
            result.Add("testfile2.txt");
            // _serviceInterface.setFolderPath(testFolder1);

            // Act
            // _serviceInterface.deleteFile("testfile3.txt");
            List<string> fileList = new List<string>();
            // fileList = _serviceInterface.getFileList();

            // Assert
            Assert.AreEqual(fileList, result);
        }

        [Test]
        public void deleteFileTestNoSuchFile()
        {
            // Assign
            List<string> result = new List<string>();
            result.Add("testfile1.txt");
            result.Add("testfile2.txt");
            result.Add("testfile3.txt");
            // _serviceInterface.setFolderPath(testFolder1);

            // Act
            // _serviceInterface.deleteFile("asd.txt");
            List<string> fileList = new List<string>();
            // fileList = _serviceInterface.getFileList();

            // Assert
            Assert.AreEqual(fileList, result);
        }

        [Test]
        public void deleteFileTestError()
        {
            // Assign
            bool catchException = false;
            // _serviceInterface.setFolderPath("asdasd");

            // Act
            try
            {
                // _serviceInterface.deleteFile("asd.txt");
            }
            catch (Exception ex)
            {
                catchException = true;
            }
            // Assert
            Assert.AreEqual(catchException, true);
        }

        [Test]
        public void deleteFileTestNotAllowed()
        {
            // Assign
            bool catchException = false;
            // _serviceInterface.setFolderPath(notAllowedFolder);

            // Act
            try
            {
                // _serviceInterface.deleteFile("asd.txt");
            }
            catch (Exception ex)
            {
                catchException = true;
            }
            // Assert
            Assert.AreEqual(catchException, true);
        }

        [Test]
        public void renameFileTestSucceed()
        {
            // Assign
            List<string> result = new List<string>();
            result.Add("testfile1.txt");
            result.Add("testfile2.txt");
            result.Add("testing.txt");
            // _serviceInterface.setFolderPath(testFolder1);

            // Act
            // _serviceInterface.renameFile("testfile3.txt", "testing.txt");
            List<string> fileList = new List<string>();
            // fileList = _serviceInterface.getFileList();

            // Assert
            Assert.AreEqual(fileList, result);
        }

        [Test]
        public void renameFileTestNoSuchFile()
        {
            // Assign
            List<string> result = new List<string>();
            result.Add("testfile1.txt");
            result.Add("testfile2.txt");
            result.Add("testfile3.txt");
            // _serviceInterface.setFolderPath(testFolder1);

            // Act
            // _serviceInterface.renameFile("asd.txt", "testing.txt");
            List<string> fileList = new List<string>();
            // fileList = _serviceInterface.getFileList();

            // Assert
            Assert.AreEqual(fileList, result);
        }

        [Test]
        public void renameFileTestError()
        {
            // Assign
            bool catchException = false;
            // _serviceInterface.setFolderPath("asdasd");

            // Act
            try
            {
                // _serviceInterface.renameFile("asd.txt");
            }
            catch (Exception ex)
            {
                catchException = true;
            }
            // Assert
            Assert.AreEqual(catchException, true);
        }

        [Test]
        public void renameFileTestNotAllowed()
        {
            // Assign
            bool catchException = false;
            // _serviceInterface.setFolderPath(notAllowedFolder);

            // Act
            try
            {
                // _serviceInterface.renameFile("asd.txt");
            }
            catch (Exception ex)
            {
                catchException = true;
            }
            // Assert
            Assert.AreEqual(catchException, true);
        }
    }
}