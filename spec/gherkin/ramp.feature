Feature: On-Ramp and Off-Ramp
  Background:
    Given the following Mus:
      | Name   | Address                                    |
      | Carlos | 0xaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa |
    And the following users:
      | Name   | Cardano Address                            |
      | Alice  | 0xbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb | 
      | Bob    | 0xcccccccccccccccccccccccccccccccccccccccc |
    And the following blockchain balances:
      | User  | Token | Balance |
      | Alice | ADA   | 1000    |
    And the following exchange rates:
      | From Token | To Token | Rate |
      | ADA        | USDb     | 0.50 |
    And Alice deposited into the contract:
      | From Token | To Token | Amount | Spend Key |
      | ADA        | USDb     | 200    | 123       |
      | ADA        |          | 50     | 456       |
    And Alice proved her Deposit to Carlos

  Scenario: Users can deposit funds in the system by minting synthetic tokens on the network
    Then Alice should have the following Mu balance:
      | Mu Name | Token | Amount |
      | Carlos  | USDb  | 200    |
      | Carlos  | ADA   | 50     |
    And Alice should have the following Cardano balance:
      | Token | Amount |
      | ADA   | 550    |

  Scenario: Users can withdraw the collateral for their synthetic
    When Alice sends a request to burn the following assets:
      | Token | Amount | Spend Key |
      | USDb  | 200    | 123       |
    And Carlos signs the request
    Then Alice should have the following Mu balance:
      | Mu Name | Token | Amount |
      | Carlos  | ADA   | 50     |
    And Alice should have the following Cardano balance:
      | Token | Amount |
      | ADA   | 950    |

  Scenario: Users can not withdraw collateral if they don't know the spend key
    When Alice sends a request to burn the following assets:
      | Token | Amount | Spend Key |
      | USDb  | 200    | abc       |
    Then Carlos should refuse to sign the transaction
    And Alice should not be able to withdraw any assets
