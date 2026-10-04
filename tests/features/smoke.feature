Feature: Cucumber harness smoke test

  Scenario: The posts API responds to a health check
    Given the posts API is running
    When Anna checks the health endpoint
    Then the response status is 200