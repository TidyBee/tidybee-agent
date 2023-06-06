namespace TidyUpSoftware.xUnitTests
{
    public class ServiceInterfaceTests
    {
        [Fact]
        public void ServiceInterfaceTestOne()
        {
            List<string> expectedEmpty = new List<string>();
            List<string> expectedFiles = new List<string>();
            expectedFiles.Add("testfile1.txt");
            expectedFiles.Add("testfile2.txt");
            expectedFiles.Add("testfile3.txt");
            expectedFiles.Add("shadow1.txt");
            List<string> expectedFolders = new List<string>();
            expectedFolders.Add("testFolder");
            expectedFolders.Add("testFolder2");
            expectedFolders.Add("testFolder3");
            // serviceInterface _serviceInterface = new ServiceInterface("testfolder1", "user");


            List<string> shadows = new List<string>();
            shadows.Add("shadow1.txt");
            shadows.Add("shadow2.txt");
            List<string> shadowsRemove = new List<string>();
            shadows.Add("shadow2.txt");
            // _serviceInterface.addShadowFiles(shadows);
            // _serviceInterface.removeShadowFiles(shadows);

            List<string> fileList = new List<string>();
            List<string> folderList = new List<string>();
            // fileList = _serviceInterface.getFileList();
            // folderList = _serviceInterface.getFolderList();
            Assert.Equal(expectedFiles, fileList);
            Assert.Equal(expectedFolders, folderList);
            // Assert.True(_serviceInterface.isTheUserAllowedHere());
            // _serviceInterface.setFolderPath("testfolder2");
            // fileList = _serviceInterface.getFileList();
            // folderList = _serviceInterface.getFolderList();
            Assert.Equal(expectedEmpty, fileList);
            Assert.Equal(expectedEmpty, folderList);
            // Assert.True(_serviceInterface.isTheUserAllowedHere());
        }

        [Fact]
        public void ServiceInterfaceTestTwo()
        {
            List<string> result = new List<string>();
            bool expected = false;
            // serviceInterface _serviceInterface = new ServiceInterface("notAllowedFolder", "user");

            List<string> shadows = new List<string>();
            shadows.Add("shadow1.txt");
            shadows.Add("shadow2.txt");
            List<string> shadowsRemove = new List<string>();
            shadows.Add("shadow2.txt");

            try {
                // _serviceInterface.addShadowFiles(shadows);
            }
            catch (Exception e)
            {
                expected = true;
            }

            List<string> list = new List<string>();
            Assert.True(expected);
            expected = false;

            try
            {
                // list = _serviceInterface.getFileList();
            }
            catch (Exception e)
            {
                expected = true;
            }

            Assert.True(expected);
            expected = false;

            try
            {
                // list = _serviceInterface.getFolderList();
            }
            catch (Exception e)
            {
                expected = true;
            }

            Assert.True(expected);
            expected = false;

            try
            {
                // list = _serviceInterface.getFileList();
            }
            catch (Exception e)
            {
                expected = true;
            }

            Assert.True(expected);
            expected = false;

            try
            {
                // list = _serviceInterface.getAccessListMember();
            }
            catch (Exception e)
            {
                expected = true;
            }

            Assert.True(expected);
            // Assert.False(_serviceInterface.isTheUserAllowedHere());
        }

        [Fact]
        public void ServiceInterfaceTestTwo()
        {
            List<string> result = new List<string>();
            bool expected = false;
            // serviceInterface _serviceInterface = new ServiceInterface("asdasd", "user");

            List<string> shadows = new List<string>();
            shadows.Add("shadow1.txt");
            shadows.Add("shadow2.txt");
            List<string> shadowsRemove = new List<string>();
            shadows.Add("shadow2.txt");

            try {
                // _serviceInterface.addShadowFiles(shadows);
            }
            catch (Exception e)
            {
                expected = true;
            }

            List<string> list = new List<string>();
            Assert.True(expected);
            expected = false;

            try
            {
                // list = _serviceInterface.getFileList();
            }
            catch (Exception e)
            {
                expected = true;
            }

            Assert.True(expected);
            expected = false;

            try
            {
                // list = _serviceInterface.getFolderList();
            }
            catch (Exception e)
            {
                expected = true;
            }

            Assert.True(expected);
            expected = false;

            try
            {
                // list = _serviceInterface.getFileList();
            }
            catch (Exception e)
            {
                expected = true;
            }

            Assert.True(expected);
            expected = false;

            try
            {
                // list = _serviceInterface.getAccessListMember();
            }
            catch (Exception e)
            {
                expected = true;
            }

            Assert.True(expected);
            expected = false;

            try
            {
                // _serviceInterface.isTheUserAllowedHere();
            }
            catch (Exception e)
            {
                expected = true;
            }

            Assert.True(expected);
        }
    }
}