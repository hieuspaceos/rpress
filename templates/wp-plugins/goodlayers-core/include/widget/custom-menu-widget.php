<?php
	/**
	 * A widget that show recent posts ( Specified by category ).
	 */

	add_action('widgets_init', 'gdlr_core_custom_menu_widget');
	if( !function_exists('gdlr_core_custom_menu_widget') ){
		function gdlr_core_custom_menu_widget() {
			register_widget( 'Goodlayers_Core_Custom_Menu_Widget' );
		}
	}

	if( !class_exists('Goodlayers_Core_Custom_Menu_Widget') ){
		class Goodlayers_Core_Custom_Menu_Widget extends WP_Widget{

			// Initialize the widget
			function __construct() {

				parent::__construct(
					'gdlr-core-custom-menu-widget', 
					esc_html__('Custom Menu Widget ( Goodlayers )', 'goodlayers-core'), 
					array('description' => esc_html__('A widget that show custom menu ( Does not support submenu level )', 'goodlayers-core'))
				);  

			}

			// Output of the widget
			function widget( $args, $instance ) {
	
				$title = empty($instance['title'])? '': apply_filters('widget_title', $instance['title']);
				$menu = empty($instance['menu'])? '': $instance['menu'];
				$style = empty($instance['style'])? 'half': $instance['style'];

				// Opening of widget
				echo gdlr_core_escape_content($args['before_widget']);
				
				// Open of title tag
				if( !empty($title) ){ 
					echo gdlr_core_escape_content($args['before_title'] . $title . $args['after_title']); 
				}
					
				// Widget Content
				$walker = apply_filters('gdlr_core_custom_menu_widget_walker', null);
				wp_nav_menu(array(
					'menu_class' => 'gdlr-core-custom-menu-widget gdlr-core-menu-style-' . $style,
					'menu' => $menu,
					'depth' => 1,
					'walker' => $walker
				));
				
				// Closing of widget
				echo gdlr_core_escape_content($args['after_widget']);
			}

			// Widget Form
			function form( $instance ) {

				$menus = wp_get_nav_menus();
				$menu_options = array();
				foreach( $menus as $menu ){
					$menu_options[$menu->term_id] = $menu->name;
				}

				if( class_exists('gdlr_core_widget_util') ){
					gdlr_core_widget_util::get_option(array(
						'title' => array(
							'type' => 'text',
							'id' => $this->get_field_id('title'),
							'name' => $this->get_field_name('title'),
							'title' => esc_html__('Title', 'goodlayers-core'),
							'value' => (isset($instance['title'])? $instance['title']: '')
						),
						'menu' => array(
							'type' => 'combobox',
							'id' => $this->get_field_id('menu'),
							'name' => $this->get_field_name('menu'),
							'title' => esc_html__('Select Menu', 'goodlayers-core'),
							'options' => $menu_options,
							'value' => (isset($instance['menu'])? $instance['menu']: '')
						),
						'style' => array(
							'type' => 'combobox',
							'id' => $this->get_field_id('style'),
							'name' => $this->get_field_name('style'),
							'title' => esc_html__('Select Style', 'goodlayers-core'),
							'options' => array(
								'half' => esc_html__('Half Menu', 'goodlayers-core'),
								'list' => esc_html__('List Menu', 'goodlayers-core'),
								'list2' => esc_html__('List 2 Menu', 'goodlayers-core'),
								'list3' => esc_html__('List 3 Menu', 'goodlayers-core'),
								'plain' => esc_html__('Plain Menu', 'goodlayers-core'),
								'box' => esc_html__('Box Menu', 'goodlayers-core'),
								'box2' => esc_html__('Box 2 Menu', 'goodlayers-core'),
							),
							'value' => (isset($instance['style'])? $instance['style']: '')
						),
					));
				}

			}
			
			// Update the widget
			function update( $new_instance, $old_instance ) {

				if( class_exists('gdlr_core_widget_util') ){
					return gdlr_core_widget_util::get_option_update($new_instance);
				}

				return $new_instance;
			}	
		} // class
	} // class_exists
?>