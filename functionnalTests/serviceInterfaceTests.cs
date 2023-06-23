using Xunit;

namespace TidyUpSoftware.xUnitTests
{
    public class ServiceInterfaceTests
    {
        [Fact]
        public void ServiceInterfaceTestOne()
        {
            Assert.Equal(1, 1);
            Assert.Equal(3, 3);
            Assert.Equal(32, 32);
            Assert.Equal(50, 50);
        }

        [Fact]
        public void ServiceInterfaceTestTwo()
        {
            Assert.True(true);
            Assert.True(true);
        }

        [Fact]
        public void ServiceInterfaceTestThree()
        {
            Assert.False(false);
        }
    }
}