Feature: Keyboard Controls
  As a widget developer
  I want vim-style and arrow key navigation
  So that I can efficiently browse ingredients without leaving the keyboard

  Background:
    Given the harness is running with ingredients loaded

  # ============================================================================
  # QUIT
  # ============================================================================

  @core @local @native @keyboard
  Scenario Outline: Quit the harness
    When the user presses "<key>"
    Then the harness should exit cleanly
    And the terminal should be restored to its original state

    Examples:
      | key |
      | q   |
      | Esc |

  # ============================================================================
  # VERTICAL MOVEMENT
  # ============================================================================

  @core @local @native @keyboard
  Scenario Outline: Move cursor down
    Given the cursor is not on the last visible entry
    When the user presses "<key>"
    Then the cursor should move down one entry

    Examples:
      | key  |
      | j    |
      | Down |

  @core @local @native @keyboard
  Scenario Outline: Move cursor up
    Given the cursor is not on the first visible entry
    When the user presses "<key>"
    Then the cursor should move up one entry

    Examples:
      | key |
      | k   |
      | Up  |

  # ============================================================================
  # EXPAND / COLLAPSE
  # ============================================================================

  @core @local @native @keyboard
  Scenario Outline: Expand a collapsed group
    Given the cursor is on a collapsed group header
    When the user presses "<key>"
    Then the group should expand to show its variants

    Examples:
      | key   |
      | l     |
      | Right |

  @core @local @native @keyboard
  Scenario Outline: Collapse an expanded group from its header
    Given the cursor is on an expanded group header
    When the user presses "<key>"
    Then the group should collapse to hide its variants

    Examples:
      | key  |
      | h    |
      | Left |

  @core @local @native @keyboard
  Scenario Outline: Collapse parent group from a variant
    Given the cursor is on a variant entry
    When the user presses "<key>"
    Then the parent group should collapse
    And the cursor should move to the parent group header

    Examples:
      | key  |
      | h    |
      | Left |

  # ============================================================================
  # TOGGLE / SELECT
  # ============================================================================

  @core @local @native @keyboard
  Scenario: Enter toggles group when cursor is on group header
    Given the cursor is on a group header
    When the user presses "Enter"
    Then the group's expansion state should toggle

  @core @local @native @keyboard
  Scenario: Enter focuses preview when cursor is on an interactive variant
    Given the cursor is on an interactive variant
    When the user presses "Enter"
    Then the preview pane should receive focus

  @core @local @native @keyboard
  Scenario: Enter is a no-op when cursor is on a non-interactive variant
    Given the cursor is on a non-interactive variant
    When the user presses "Enter"
    Then the cursor should remain on the same entry
    And no expansion state should change

  # ============================================================================
  # PREVIEW FOCUS MODE
  # ============================================================================

  @core @local @native @keyboard
  Scenario: Esc exits preview focus
    Given the preview pane is focused
    When the user presses "Esc"
    Then the focus should return to the sidebar

  @core @local @native @keyboard
  Scenario: q does not quit while preview is focused
    Given the preview pane is focused
    When the user presses "q"
    Then the harness should continue running
    And the key should be forwarded to the ingredient

  @core @local @native @keyboard
  Scenario Outline: Navigation keys forward to ingredient in preview focus
    Given the preview pane is focused on an interactive ingredient
    When the user presses "<key>"
    Then the ingredient should receive the key event
    And the sidebar should not respond

    Examples:
      | key  |
      | j    |
      | k    |
      | Up   |
      | Down |

  # ============================================================================
  # EDGE CASES
  # ============================================================================

  @core @local @native @keyboard
  Scenario: Unbound keys are ignored
    When the user presses an unbound key
    Then no state should change
    And the harness should continue running

  @core @local @native @keyboard
  Scenario: Only key press events are handled
    When a key release or repeat event occurs
    Then the harness should ignore it
