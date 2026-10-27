<?php
	/**
	 * A widget that show recent posts ( Specified by category ).
	 */

	add_action('widgets_init', 'gdlr_core_newsletter_widget');
	if( !function_exists('gdlr_core_newsletter_widget') ){
		function gdlr_core_newsletter_widget() {
			register_widget( 'Goodlayers_Core_Newsletter_Widget' );
		}
	}

	if( !class_exists('Goodlayers_Core_Newsletter_Widget') ){
		class Goodlayers_Core_Newsletter_Widget extends WP_Widget{

			// Initialize the widget
			function __construct() {

				parent::__construct(
					'gdlr-core-newsletter-widget', 
					esc_html__('Newsletter Widget ( Goodlayers )', 'goodlayers-core'), 
					array('description' => esc_html__('Have to install the "Newsletter" plugin first.', 'goodlayers-core'))
				);  

			}

			// Output of the widget
			function widget( $args, $instance ) {
	
				$title = empty($instance['title'])? '': apply_filters('widget_title', $instance['title']);
				$style = empty($instance['style'])? 'style-1': $instance['style'];
				$icon_color = empty($instance['icon-color'])? '#ffc327': $instance['icon-color'];
				$bg_color = empty($instance['bg-color'])? '': $instance['bg-color'];
					
				// Opening of widget
				echo gdlr_core_escape_content($args['before_widget']);
				
				// Open of title tag
				if( !empty($title) ){ 
					echo gdlr_core_escape_content($args['before_title'] . $title . $args['after_title']); 
				}
					
				// Widget Content
				if( !class_exists('NewsletterSubscription') ){
					echo wp_kses(__('Please install and activate the "<a target="_blank" href="https://wordpress.org/plugins/newsletter/" >Newsletter</a>" plugin to show the form.', 'goodlayers-core'), 
						array( 'a' => array('target'=>array(), 'href'=>array()) ));
				}else{
?>
<div class="gdlr-core-with-fa-send-o-button tnp tnp-subscription gdlr-core-<?php echo esc_attr($style); ?>">
<form method="post" action="<?php echo esc_attr(home_url('/') . '?na=s'); ?>" onsubmit="return newsletter_check(this)">

<input type="hidden" name="nlang" value="" >
<div class="tnp-field tnp-field-email">
	<input class="tnp-email" type="email" name="ne" placeholder="<?php esc_html_e('Enter Your Email Address', 'goodlayers-core'); ?>" required <?php echo empty($bg_color)? '': 'style="background: ' . $bg_color . '"'; ?> >
</div>
<?php 
	if( $style == 'style-1' ){
		echo '<div class="tnp-field tnp-field-button" style="color: ' . $icon_color . ';" >';
		echo '<input class="tnp-submit" type="submit" value="Subscribe">';
		echo '</div>';
	}else if( $style == 'style-2' ){
		echo '<div class="tnp-field tnp-field-button" style="color: #fff;" >';
		echo '<input class="tnp-submit" type="submit" value="Subscribe" style="background-color:' . $icon_color . ';" >';
		echo '</div>';
	}
?>
</form>
</div>
<?php
				}

				// Closing of widget
				echo gdlr_core_escape_content($args['after_widget']);

			}

			// Widget Form
			function form( $instance ) {

				if( class_exists('gdlr_core_widget_util') ){
					gdlr_core_widget_util::get_option(array(
						'title' => array(
							'type' => 'text',
							'id' => $this->get_field_id('title'),
							'name' => $this->get_field_name('title'),
							'title' => esc_html__('Title', 'goodlayers-core'),
							'value' => (isset($instance['title'])? $instance['title']: '')
						),
						'style' => array(
							'type' => 'combobox',
							'options' => array(
								'style-1' => esc_html__('Style 1', 'goodlayers-core'),
								'style-2' => esc_html__('Style 2', 'goodlayers-core'),
							),
							'id' => $this->get_field_id('style'),
							'name' => $this->get_field_name('style'),
							'title' => esc_html__('Style', 'goodlayers-core'),
							'value' => (isset($instance['style'])? $instance['style']: 'style-1')
						),
						'icon-color' => array(
							'type' => 'colorpicker',
							'id' => $this->get_field_id('icon-color'),
							'name' => $this->get_field_name('icon-color'),
							'title' => esc_html__('Icon Color', 'goodlayers-core'),
							'value' => (isset($instance['icon-color'])? $instance['icon-color']: '#ffc327')
						),
						'bg-color' => array(
							'type' => 'colorpicker',
							'id' => $this->get_field_id('bg-color'),
							'name' => $this->get_field_name('bg-color'),
							'title' => esc_html__('Background Color', 'goodlayers-core'),
							'value' => (isset($instance['bg-color'])? $instance['bg-color']: '')
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