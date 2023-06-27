using NUnit.Framework;

namespace TidyUpSoftware.nUnitTests
{
    public class BasicTests
    {
        [SetUp]
        public void Setup()
        {
        }

        [Test]
        public void test1()
        {
            Assert.That(1, Is.EqualTo(1));
        }

        [Test]
        public void test2()
        {
            Assert.False(false);
        }

        [Test]
        public void test3()
        {
            Assert.True(true);
        }

        [Test]
        public void test4()
        {
            Assert.IsTrue(true);
        }

        [Test]
        public void test5()
        {
            Assert.That(1, Is.Not.EqualTo(2));
        }

        [Test]
        public void test6()
        {
            Assert.That(1, Is.LessThan(2));
        }

        [Test]
        public void test7()
        {
            Assert.That(1, Is.LessThanOrEqualTo(1));
        }

        [Test]
        public void test8()
        {
            Assert.That(10, Is.GreaterThan(1));
        }

        [Test]
        public void test9()
        {
            Assert.That(20, Is.GreaterThanOrEqualTo(20));
        }
    }
}