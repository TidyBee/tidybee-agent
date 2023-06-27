using Xunit;

namespace TidyUpSoftware.xUnitTests
{
    public class BasicTests
    {
        [Fact]
        public void TestOne()
        {
            Assert.Equal(1, 1);
            Assert.Equal(3, 3);
            Assert.Equal(32, 32);
            Assert.Equal(50, 50);
        }

        [Fact]
        public void TestTwo()
        {
            Assert.True(true);
            Assert.True(true);
        }

        [Fact]
        public void TestThree()
        {
            Assert.False(false);
        }
    }
}