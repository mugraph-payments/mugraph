Feature: Transfers between users in the same Mu
  Background:
    Given the following Mus:
      | Name   | Address                                    |
      | Carlos | 0xaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa |
    And the following users:
      | Name   | Cardano Address                            |
      | Alice  | 0xbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb | 
      | Bob    | 0xcccccccccccccccccccccccccccccccccccccccc |
    And the following Mu Balances:
      | Mu Name | User   | Token | Amount |
      | Carlos  | Alice  | USDb  | 200    |
      | Carlos  | Alice  | ADA   | 50     |

  Scenario: Users can withdraw the collateral for their synthetic
    When Bob sends Alice his new spend keys for the assets
    And Bob sends Alice a notify key
    And Alice sends a Delta to Carlos:
      | Token | Amount | Spend Key | New Spend Key |
      | USDb  | 200    | 123       | 987           |
      | ADA   | 10     | 456       | 654           |
    Then Carlos signs the Delta Proof
    And Alice sends the signed transaction to Bob
    And Alice should have the following Mu balance:
      | Mu Name | Token | Amount |
      | Carlos  | ADA   | 40     |
    And Bob should have the following Mu balance:
      | Mu Name | Token | Amount |
      | Carlos  | ADA   | 10     |
      | Carlos  | USDb  | 50     |

Feature: Cross-Mu Transfers
  Background:
    Given the following Mus:
      | Name   | Address                                    |
      | Carlos | 0xaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa |
      | Dave   | 0xdddddddddddddddddddddddddddddddddddddddd |
    And the following users:
      | Name   | Cardano Address                            |
      | Alice  | 0xbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb | 
      | Bob    | 0xcccccccccccccccccccccccccccccccccccccccc |
    And the following Mu Balances:
      | Mu Name | User   | Token | Amount |
      | Carlos  | Alice  | USDb  | 200    |
      | Dave    | Alice  | ADA   | 50     |

  Scenario: User consolidates their assets in a single mu
    When Alice sends a Delta to Dave:
      | Asset | Amount | Spend Key | New Spend Key | New Mu   | 
      | ADA   | 50     | 456       | 654           | 0xaaa... |
    And Dave signs the Delta
    And Carlos signs the delta
    And the Mus should have the following balances:
      | Mu Name | User   | Token | Amount |
      | Carlos  | Alice  | USDb  | 200    |
      | Carlos  | Alice  | ADA   | 50     |
