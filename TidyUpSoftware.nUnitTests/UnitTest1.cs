namespace TidyUpSoftware.nUnitTests
{
    public class Tests
    {
        [SetUp]
        public void Setup()
        {
        }

        [Test]
        public void Test1()
        {
            // Assign
            int result = 3;

            // Act
            List<string> duplicates = new List<string>();

            // Assert
            Assert.Equals(duplicates.Count, result);
        }
    }
}