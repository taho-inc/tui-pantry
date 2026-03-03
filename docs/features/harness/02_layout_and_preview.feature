Feature: Two-Pane Layout and Preview
  As a widget developer
  I want a split layout with sidebar navigation and a live preview pane
  So that I can see my widget rendered in isolation as I browse

  Background:
    Given the harness is loaded with ingredients
    And a variant is selected in the navigation tree

  # ============================================================================
  # LAYOUT STRUCTURE
  # ============================================================================

  @core @local @native @rendering
  Scenario: Layout splits into sidebar and preview
    Then the left region should display the navigation sidebar
    And the right region should display the ingredient preview
    And the sidebar should be separated from the preview by a vertical border

  @core @local @native @rendering
  Scenario: Key hints bar appears at the bottom
    Then a key hints bar should appear below the main content
    And the hints should include "navigate", "expand", "collapse", "select", and "quit"
    And the hint keys should be visually distinct from their labels

  @core @local @native @rendering
  Scenario: Purple gradient background frames the layout
    Then the terminal background should display a purple gradient
    And the main content panels should sit inset from the terminal edges

  # ============================================================================
  # PREVIEW PANE
  # ============================================================================

  @core @local @native @rendering
  Scenario: Preview renders selected ingredient in a bordered pane
    Then the preview area should display the selected ingredient
    And the ingredient should render inside a titled border
    And the border title should contain the ingredient's name

  @core @local @native @rendering
  Scenario: Preview shows breadcrumb with group, name, and source
    Then the breadcrumb should display the ingredient's group
    And the breadcrumb should display a separator
    And the breadcrumb should display the ingredient's name
    And the breadcrumb should display the ingredient's source module path

  @core @local @native @rendering
  Scenario: Preview updates when a different variant is selected
    When the user navigates the cursor to a different variant
    Then the preview pane should render the newly selected ingredient
    And the breadcrumb should reflect the new ingredient's metadata

  @core @local @native @rendering
  Scenario: Preview shows placeholder when no variant is selected
    Given the cursor is on a group header
    Then the preview pane should display "Select an ingredient from the sidebar"

  # ============================================================================
  # INGREDIENT RENDERING CONTRACT
  # ============================================================================

  @core @local @native @rendering
  Scenario: Ingredient receives the full inner area of the pane
    Then the ingredient's render method should receive the area inside the pane border
    And the ingredient should not need to account for its own chrome

  @core @local @native @rendering
  Scenario: Multiple ingredients from the same group render independently
    Given two variants exist in the same group
    When the user switches between them
    Then each variant should render its own distinct content
    And neither variant's state should affect the other

  # ============================================================================
  # EDGE CASES
  # ============================================================================

  @core @local @native @rendering
  Scenario: Sidebar truncates entries that exceed its width
    Given an ingredient name is longer than the sidebar width
    Then the sidebar should truncate the name to fit within its bounds

  @core @local @native @rendering
  Scenario: Sidebar scrolls when entries exceed visible height
    Given more entries are visible than fit in the sidebar height
    Then entries beyond the visible area should not render
    But the cursor should still be able to reach all entries
