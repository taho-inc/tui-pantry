Feature: Semantic Color System
  As a widget developer
  I want a semantic color system with named roles
  So that widgets use consistent colors and themes can be swapped

  Background:
    Given the taho theme is loaded

  # ============================================================================
  # PALETTE STRUCTURE
  # ============================================================================

  @core @local @native @rendering
  Scenario: Brand palette provides core identity colors
    Then the brand palette should contain "Deep Purple" as #2E1574
    And the brand palette should contain "Vivid Purple" as #7834F5
    And the brand palette should contain "White" as #FFFFFF
    And the brand palette should contain "Black" as #000000

  @core @local @native @rendering
  Scenario: Accent palette provides decorative colors
    Then the accent palette should contain 5 colors
    And each accent color should be a distinct RGB value

  @core @local @native @rendering
  Scenario: State palette provides semantic indicator colors
    Then the state palette should contain "OK" as #4ADE80
    And the state palette should contain "Warn" as #FBBF24
    And the state palette should contain "Critical" as #FB7185
    And the state palette should contain "Info" as #44BBA4

  @core @local @native @rendering
  Scenario: Surface palette provides background and border colors
    Then the surface palette should contain "Panel" as RGB(13, 13, 13)
    And the surface palette should contain "Panel Raised" as RGB(26, 26, 26)
    And the surface palette should contain "Border"
    And the surface palette should contain "Cursor BG"

  # ============================================================================
  # THEME ROLES
  # ============================================================================

  @core @local @native @rendering
  Scenario: Theme maps semantic roles to palette colors
    Then the theme should have a "primary" role
    And the theme should have an "accent" role
    And the theme should have a "text" role
    And the theme should have an "ok" role
    And the theme should have a "warn" role
    And the theme should have a "critical" role

  @core @local @native @rendering
  Scenario: Theme provides text hierarchy with three levels
    Then the theme "text" color should be white
    And the theme "text_dim" color should be dark gray
    And the theme "text_disabled" color should be dimmer than "text_dim"

  @core @local @native @rendering
  Scenario: Theme maps ratio thresholds to state colors
    When a ratio of 0.50 is mapped through the theme
    Then the result should be the "ok" color
    When a ratio of 0.75 is mapped through the theme
    Then the result should be the "warn" color
    When a ratio of 0.90 is mapped through the theme
    Then the result should be the "critical" color

  # ============================================================================
  # WIDGET INTEGRATION
  # ============================================================================

  @core @local @native @rendering
  Scenario: Widgets render using theme colors not hardcoded values
    Given a NodeTable widget rendered with the taho theme
    Then the header should use the theme "text_dim" color
    And the border should use the theme "border" color
    And online nodes should use the theme "ok" color
    And degraded nodes should use the theme "warn" color
    And offline nodes should use the theme "critical" color

  @core @local @native @rendering
  Scenario: ResourceGauge uses theme state colors for threshold bands
    Given a ResourceGauge rendered at 50% with the taho theme
    Then the gauge color should be the theme "ok" color
    Given a ResourceGauge rendered at 75% with the taho theme
    Then the gauge color should be the theme "warn" color
    Given a ResourceGauge rendered at 90% with the taho theme
    Then the gauge color should be the theme "critical" color

  @core @local @native @rendering
  Scenario: EventList severity maps to theme state colors
    Given an EventList with mixed severity entries rendered with the taho theme
    Then info entries should use the theme "info" color
    And warn entries should use the theme "warn" color
    And error entries should use the theme "critical" color

  # ============================================================================
  # THEMING
  # ============================================================================

  @core @local @native
  Scenario: Theme is const-constructable for static usage
    Given the taho theme is constructed with Theme::taho()
    Then it should be usable as a const value

  @core @local @native
  Scenario: Theme implements Default
    Given a Theme created via Default::default()
    Then it should equal Theme::taho()
